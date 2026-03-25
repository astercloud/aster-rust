import { Sliders, Bot } from 'lucide-react';
import React, { useEffect, useState } from 'react';
import { useModelAndProvider } from '../../../ModelAndProviderContext';
import { SwitchModelModal } from '../subcomponents/SwitchModelModal';
import { LeadWorkerSettings } from '../subcomponents/LeadWorkerSettings';
import { View } from '../../../../utils/navigationUtils';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../../../ui/dropdown-menu';
import { useCurrentModelInfo } from '../../../../contexts/SessionExecutionContext';
import { useConfig } from '../../../ConfigContext';
import { getProviderMetadata } from '../modelInterface';
import { Alert } from '../../../alerts';
import BottomMenuAlertPopover from '../../../bottom_menu/BottomMenuAlertPopover';
import { getModelDisplayName, getProviderDisplayName } from '../predefinedModelsUtils';
import { getOutputSchemaRuntimeLabel } from '../../../../utils/sessionExecutionRuntime';

interface ModelsBottomBarProps {
  sessionId: string | null;
  dropdownRef: React.RefObject<HTMLDivElement>;
  setView: (view: View) => void;
  alerts: Alert[];
  preferRuntime?: boolean;
}

export default function ModelsBottomBar({
  sessionId,
  dropdownRef,
  setView,
  alerts,
  preferRuntime = false,
}: ModelsBottomBarProps) {
  const { currentModel, currentProvider } = useModelAndProvider();
  const currentModelInfo = useCurrentModelInfo();
  const { read, getProviders } = useConfig();
  const effectiveModel =
    preferRuntime && currentModelInfo?.model ? currentModelInfo.model : currentModel;
  const effectiveProvider =
    preferRuntime && currentModelInfo?.provider ? currentModelInfo.provider : currentProvider;
  const [displayProvider, setDisplayProvider] = useState<string | null>(null);
  const [displayModelName, setDisplayModelName] = useState<string>('Select Model');
  const [isAddModelModalOpen, setIsAddModelModalOpen] = useState(false);
  const [isLeadWorkerModalOpen, setIsLeadWorkerModalOpen] = useState(false);
  const [isLeadWorkerActive, setIsLeadWorkerActive] = useState(false);
  const [providerDefaultModel, setProviderDefaultModel] = useState<string | null>(null);

  // Check if lead/worker mode is active
  useEffect(() => {
    const checkLeadWorker = async () => {
      try {
        const leadModel = await read('ASTER_LEAD_MODEL', false);
        setIsLeadWorkerActive(!!leadModel);
      } catch (error) {
        console.error('Error checking lead model:', error);
        setIsLeadWorkerActive(false);
      }
    };
    checkLeadWorker();
  }, [read]);

  // Refresh lead/worker status when modal closes
  const handleLeadWorkerModalClose = () => {
    setIsLeadWorkerModalOpen(false);
    // Refresh the lead/worker status after modal closes
    const checkLeadWorker = async () => {
      try {
        const leadModel = await read('ASTER_LEAD_MODEL', false);
        const currentModel = await read('ASTER_MODEL', false);
        setIsLeadWorkerActive(!!leadModel);
        setLeadModelName((leadModel as string) || '');
        setCurrentActiveModel((currentModel as string) || '');
      } catch (error) {
        console.error('Error checking lead model after modal close:', error);
        setIsLeadWorkerActive(false);
      }
    };
    checkLeadWorker();
  };

  const [leadModelName, setLeadModelName] = useState<string>('');
  const [currentActiveModel, setCurrentActiveModel] = useState<string>('');

  // Get lead model name and current model for comparison
  useEffect(() => {
    const getModelInfo = async () => {
      try {
        const leadModel = await read('ASTER_LEAD_MODEL', false);
        const currentModel = await read('ASTER_MODEL', false);
        setLeadModelName((leadModel as string) || '');
        setCurrentActiveModel((currentModel as string) || '');
      } catch (error) {
        console.error('Error getting model info:', error);
      }
    };
    getModelInfo();
  }, [read]);

  const fallbackMode = isLeadWorkerActive
    ? currentActiveModel === leadModelName
      ? 'lead'
      : 'worker'
    : undefined;
  const modelMode = preferRuntime ? currentModelInfo?.mode || fallbackMode : fallbackMode;

  const displayModel = effectiveModel || providerDefaultModel || displayModelName;
  const outputSchemaLabel = getOutputSchemaRuntimeLabel(currentModelInfo?.outputSchemaRuntime);

  // Resolve display provider from the effective runtime first, then fall back to provider metadata.
  useEffect(() => {
    if (!effectiveProvider && !effectiveModel) {
      setDisplayProvider(null);
      return;
    }

    (async () => {
      const providerDisplayNameFromModel = effectiveModel
        ? getProviderDisplayName(effectiveModel)
        : '';
      if (providerDisplayNameFromModel) {
        setDisplayProvider(providerDisplayNameFromModel);
        return;
      }

      if (!effectiveProvider) {
        setDisplayProvider(null);
        return;
      }

      try {
        const metadata = await getProviderMetadata(effectiveProvider, getProviders);
        setDisplayProvider(metadata.display_name);
      } catch (error) {
        console.error('Failed to resolve provider display name:', error);
        setDisplayProvider(effectiveProvider);
      }
    })();
  }, [effectiveProvider, effectiveModel, getProviders]);

  // Fetch provider default model when provider changes and no effective model is present.
  useEffect(() => {
    if (effectiveProvider && !effectiveModel) {
      (async () => {
        try {
          const metadata = await getProviderMetadata(effectiveProvider, getProviders);
          setProviderDefaultModel(metadata.default_model);
        } catch (error) {
          console.error('Failed to get provider default model:', error);
          setProviderDefaultModel(null);
        }
      })();
    } else if (effectiveModel) {
      setProviderDefaultModel(null);
    }
  }, [effectiveProvider, effectiveModel, getProviders]);

  // The visible label should reflect the effective execution model when available.
  useEffect(() => {
    setDisplayModelName(effectiveModel ? getModelDisplayName(effectiveModel) : 'Select Model');
  }, [effectiveModel]);

  return (
    <div className="relative flex items-center" ref={dropdownRef}>
      <BottomMenuAlertPopover alerts={alerts} />
      <DropdownMenu>
        <DropdownMenuTrigger className="flex items-center hover:cursor-pointer max-w-[180px] md:max-w-[200px] lg:max-w-[380px] min-w-0 text-text-default/70 hover:text-text-default transition-colors">
          <div className="flex items-center truncate max-w-[130px] md:max-w-[200px] lg:max-w-[360px] min-w-0">
            <Bot className="mr-1 h-4 w-4 flex-shrink-0" />
            <span className="truncate text-xs">
              {displayModel}
              {modelMode && <span className="ml-1 text-[10px] opacity-60">({modelMode})</span>}
            </span>
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent side="top" align="center" className="w-64 text-sm">
          <h6 className="text-xs text-textProminent mt-2 ml-2">Current model</h6>
          <p className="flex items-center justify-between text-sm mx-2 pb-2 border-b mb-2">
            {displayModelName}
            {displayProvider && ` — ${displayProvider}`}
          </p>
          {outputSchemaLabel ? (
            <p className="mx-2 -mt-1 mb-2 text-[11px] text-text-muted">{outputSchemaLabel}</p>
          ) : null}
          <DropdownMenuItem onClick={() => setIsAddModelModalOpen(true)}>
            <span>Change Model</span>
            <Sliders className="ml-auto h-4 w-4 rotate-90" />
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => setIsLeadWorkerModalOpen(true)}>
            <span>Lead/Worker Settings</span>
            <Sliders className="ml-auto h-4 w-4" />
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      {isAddModelModalOpen ? (
        <SwitchModelModal
          sessionId={sessionId}
          setView={setView}
          onClose={() => setIsAddModelModalOpen(false)}
        />
      ) : null}

      {isLeadWorkerModalOpen ? (
        <LeadWorkerSettings isOpen={isLeadWorkerModalOpen} onClose={handleLeadWorkerModalClose} />
      ) : null}
    </div>
  );
}
