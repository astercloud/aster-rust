import AsterLogo from './AsterLogo';
import AnimatedIcons from './AnimatedIcons';
import FlyingBird from './FlyingBird';
import { ChatState } from '../types/chatState';

interface LoadingAsterProps {
  message?: string;
  chatState?: ChatState;
}

const STATE_MESSAGES: Record<ChatState, string> = {
  [ChatState.LoadingConversation]: 'loading conversation...',
  [ChatState.Thinking]: 'aster is thinking…',
  [ChatState.Streaming]: 'aster is working on it…',
  [ChatState.WaitingForUserInput]: 'aster is waiting…',
  [ChatState.Compacting]: 'aster is compacting the conversation...',
  [ChatState.Idle]: 'aster is working on it…',
};

const STATE_ICONS: Record<ChatState, React.ReactNode> = {
  [ChatState.LoadingConversation]: <AnimatedIcons className="flex-shrink-0" cycleInterval={600} />,
  [ChatState.Thinking]: <AnimatedIcons className="flex-shrink-0" cycleInterval={600} />,
  [ChatState.Streaming]: <FlyingBird className="flex-shrink-0" cycleInterval={150} />,
  [ChatState.WaitingForUserInput]: (
    <AnimatedIcons className="flex-shrink-0" cycleInterval={600} variant="waiting" />
  ),
  [ChatState.Compacting]: <AnimatedIcons className="flex-shrink-0" cycleInterval={600} />,
  [ChatState.Idle]: <AsterLogo size="small" hover={false} />,
};

const LoadingAster = ({ message, chatState = ChatState.Idle }: LoadingAsterProps) => {
  const displayMessage = message || STATE_MESSAGES[chatState];
  const icon = STATE_ICONS[chatState];

  return (
    <div className="w-full animate-fade-slide-up">
      <div
        data-testid="loading-indicator"
        className="flex items-center gap-2 text-xs text-textStandard py-2"
      >
        {icon}
        {displayMessage}
      </div>
    </div>
  );
};

export default LoadingAster;
