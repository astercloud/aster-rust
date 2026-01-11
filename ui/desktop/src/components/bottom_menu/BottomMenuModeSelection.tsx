import { useCallback, useEffect, useState } from 'react';
import { Tornado } from 'lucide-react';
import { all_aster_modes, ModeSelectionItem } from '../settings/mode/ModeSelectionItem';
import { useConfig } from '../ConfigContext';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu';
import { trackModeChanged } from '../../utils/analytics';

export const BottomMenuModeSelection = () => {
  const [asterMode, setAsterMode] = useState('auto');
  const { read, upsert } = useConfig();

  const fetchCurrentMode = useCallback(async () => {
    try {
      const mode = (await read('ASTER_MODE', false)) as string;
      if (mode) {
        setAsterMode(mode);
      }
    } catch (error) {
      console.error('Error fetching current mode:', error);
    }
  }, [read]);

  useEffect(() => {
    fetchCurrentMode();
  }, [fetchCurrentMode]);

  const handleModeChange = async (newMode: string) => {
    if (asterMode === newMode) {
      return;
    }

    try {
      await upsert('ASTER_MODE', newMode, false);
      setAsterMode(newMode);
      trackModeChanged(asterMode, newMode);
    } catch (error) {
      console.error('Error updating aster mode:', error);
      throw new Error(`Failed to store new aster mode: ${newMode}`);
    }
  };

  function getValueByKey(key: string) {
    const mode = all_aster_modes.find((mode) => mode.key === key);
    return mode ? mode.label : 'auto';
  }

  function getModeDescription(key: string) {
    const mode = all_aster_modes.find((mode) => mode.key === key);
    return mode ? mode.description : 'Automatic mode selection';
  }

  return (
    <div title={`Current mode: ${getValueByKey(asterMode)} - ${getModeDescription(asterMode)}`}>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <span className="flex items-center cursor-pointer [&_svg]:size-4 text-text-default/70 hover:text-text-default hover:scale-100 hover:bg-transparent text-xs">
            <Tornado className="mr-1 h-4 w-4" />
            {getValueByKey(asterMode).toLowerCase()}
          </span>
        </DropdownMenuTrigger>
        <DropdownMenuContent className="w-64" side="top" align="center">
          {all_aster_modes.map((mode) => (
            <DropdownMenuItem key={mode.key} asChild>
              <ModeSelectionItem
                mode={mode}
                currentMode={asterMode}
                showDescription={false}
                isApproveModeConfigure={false}
                handleModeChange={handleModeChange}
              />
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};
