import Electron from 'electron';
import fs from 'node:fs';
import { spawn, ChildProcess } from 'child_process';
import { createServer } from 'net';
import os from 'node:os';
import path from 'node:path';
import log from './utils/logger';
import { App } from 'electron';
import { Buffer } from 'node:buffer';

import { status } from './api';
import { Client } from './api/client';
import { ExternalAsterdConfig } from './utils/settings';

export const findAvailablePort = (): Promise<number> => {
  return new Promise((resolve, _reject) => {
    const server = createServer();

    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address() as { port: number };
      server.close(() => {
        log.info(`Found available port: ${port}`);
        resolve(port);
      });
    });
  });
};

// Check if asterd server is ready by polling the status endpoint
export const checkServerStatus = async (client: Client, errorLog: string[]): Promise<boolean> => {
  const interval = 100; // ms
  const maxAttempts = 100; // 10s

  const fatal = (line: string) => {
    const trimmed = line.trim().toLowerCase();
    return trimmed.startsWith("thread 'main' panicked at") || trimmed.startsWith('error:');
  };

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    if (errorLog.some(fatal)) {
      log.error('Detected fatal error in server logs');
      return false;
    }
    try {
      await status({ client, throwOnError: true });
      return true;
    } catch {
      if (attempt === maxAttempts) {
        log.error(`Server failed to respond after ${(interval * maxAttempts) / 1000} seconds`);
      }
    }
    await new Promise((resolve) => setTimeout(resolve, interval));
  }
  return false;
};

export interface AsterdResult {
  baseUrl: string;
  workingDir: string;
  process: ChildProcess;
  errorLog: string[];
}

const connectToExternalBackend = (workingDir: string, url: string): AsterdResult => {
  log.info(`Using external asterd backend at ${url}`);

  const mockProcess = {
    pid: undefined,
    kill: () => {
      log.info(`Not killing external process that is managed externally`);
    },
  } as ChildProcess;

  return { baseUrl: url, workingDir, process: mockProcess, errorLog: [] };
};

interface AsterProcessEnv {
  [key: string]: string | undefined;

  HOME: string;
  USERPROFILE: string;
  APPDATA: string;
  LOCALAPPDATA: string;
  PATH: string;
  ASTER_PORT: string;
  ASTER_SERVER__SECRET_KEY?: string;
}

export interface StartAsterdOptions {
  app: App;
  serverSecret: string;
  dir: string;
  env?: Partial<AsterProcessEnv>;
  externalAsterd?: ExternalAsterdConfig;
}

export const startAsterd = async (options: StartAsterdOptions): Promise<AsterdResult> => {
  const { app, serverSecret, dir: inputDir, env = {}, externalAsterd } = options;
  const isWindows = process.platform === 'win32';
  const homeDir = os.homedir();
  const dir = path.resolve(path.normalize(inputDir));

  if (externalAsterd?.enabled && externalAsterd.url) {
    return connectToExternalBackend(dir, externalAsterd.url);
  }

  if (process.env.ASTER_EXTERNAL_BACKEND) {
    return connectToExternalBackend(dir, 'http://127.0.0.1:3000');
  }

  let asterdPath = getAsterdBinaryPath(app);

  const resolvedAsterdPath = path.resolve(asterdPath);

  const port = await findAvailablePort();
  const stderrLines: string[] = [];

  log.info(`Starting asterd from: ${resolvedAsterdPath} on port ${port} in dir ${dir}`);

  const additionalEnv: AsterProcessEnv = {
    HOME: homeDir,
    USERPROFILE: homeDir,
    APPDATA: process.env.APPDATA || path.join(homeDir, 'AppData', 'Roaming'),
    LOCALAPPDATA: process.env.LOCALAPPDATA || path.join(homeDir, 'AppData', 'Local'),
    PATH: `${path.dirname(resolvedAsterdPath)}${path.delimiter}${process.env.PATH || ''}`,
    ASTER_PORT: String(port),
    ASTER_SERVER__SECRET_KEY: serverSecret,
    ...env,
  } as AsterProcessEnv;

  const processEnv: AsterProcessEnv = { ...process.env, ...additionalEnv } as AsterProcessEnv;

  if (isWindows && !resolvedAsterdPath.toLowerCase().endsWith('.exe')) {
    asterdPath = resolvedAsterdPath + '.exe';
  } else {
    asterdPath = resolvedAsterdPath;
  }
  log.info(`Binary path resolved to: ${asterdPath}`);

  const spawnOptions = {
    cwd: dir,
    env: processEnv,
    stdio: ['ignore', 'pipe', 'pipe'] as ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
    detached: isWindows,
    shell: false,
  };

  const safeSpawnOptions = {
    ...spawnOptions,
    env: Object.keys(spawnOptions.env || {}).reduce(
      (acc, key) => {
        if (key.includes('SECRET') || key.includes('PASSWORD') || key.includes('TOKEN')) {
          acc[key] = '[REDACTED]';
        } else {
          acc[key] = spawnOptions.env![key] || '';
        }
        return acc;
      },
      {} as Record<string, string>
    ),
  };
  log.info('Spawn options:', JSON.stringify(safeSpawnOptions, null, 2));

  const safeArgs = ['agent'];

  const asterdProcess: ChildProcess = spawn(asterdPath, safeArgs, spawnOptions);

  if (isWindows && asterdProcess.unref) {
    asterdProcess.unref();
  }

  asterdProcess.stdout?.on('data', (data: Buffer) => {
    log.info(`asterd stdout for port ${port} and dir ${dir}: ${data.toString()}`);
  });

  asterdProcess.stderr?.on('data', (data: Buffer) => {
    const lines = data
      .toString()
      .split('\n')
      .filter((l) => l.trim());
    lines.forEach((line) => {
      log.error(`asterd stderr for port ${port} and dir ${dir}: ${line}`);
      stderrLines.push(line);
    });
  });

  asterdProcess.on('close', (code: number | null) => {
    log.info(`asterd process exited with code ${code} for port ${port} and dir ${dir}`);
  });

  asterdProcess.on('error', (err: Error) => {
    log.error(`Failed to start asterd on port ${port} and dir ${dir}`, err);
    throw err;
  });

  const try_kill_aster = () => {
    try {
      if (isWindows) {
        const pid = asterdProcess.pid?.toString() || '0';
        spawn('taskkill', ['/pid', pid, '/T', '/F'], { shell: false });
      } else {
        asterdProcess.kill?.();
      }
    } catch (error) {
      log.error('Error while terminating asterd process:', error);
    }
  };

  app.on('will-quit', () => {
    log.info('App quitting, terminating asterd server');
    try_kill_aster();
  });

  log.info(`Asterd server successfully started on port ${port}`);
  return {
    baseUrl: `http://127.0.0.1:${port}`,
    workingDir: dir,
    process: asterdProcess,
    errorLog: stderrLines,
  };
};

const getAsterdBinaryPath = (app: Electron.App): string => {
  let executableName = process.platform === 'win32' ? 'asterd.exe' : 'asterd';

  let possiblePaths: string[];
  if (!app.isPackaged) {
    possiblePaths = [
      path.join(process.cwd(), 'src', 'bin', executableName),
      path.join(process.cwd(), 'bin', executableName),
      path.join(process.cwd(), '..', '..', 'target', 'debug', executableName),
      path.join(process.cwd(), '..', '..', 'target', 'release', executableName),
    ];
  } else {
    possiblePaths = [path.join(process.resourcesPath, 'bin', executableName)];
  }

  for (const binPath of possiblePaths) {
    try {
      const resolvedPath = path.resolve(binPath);

      if (fs.existsSync(resolvedPath)) {
        const stats = fs.statSync(resolvedPath);
        if (stats.isFile()) {
          return resolvedPath;
        } else {
          log.error(`Path exists but is not a regular file: ${resolvedPath}`);
        }
      }
    } catch (error) {
      log.error(`Error checking path ${binPath}:`, error);
    }
  }

  throw new Error(
    `Could not find ${executableName} binary in any of the expected locations: ${possiblePaths.join(
      ', '
    )}`
  );
};
