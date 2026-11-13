import type { StatusTone } from '@nakama/api';
import { Badge } from '@nakama/ui/components';

const toneVariants = {
  ok: 'default',
  busy: 'outline',
  idle: 'secondary',
  error: 'destructive',
} as const;

export type StatusBadgeProps = {
  tone: StatusTone;
  children: string;
};

export function StatusBadge({ tone, children }: StatusBadgeProps) {
  return (
    <Badge variant={toneVariants[tone]}>
      <span aria-hidden className={`size-1.5 rounded-full bg-current ${tone === 'busy' ? 'animate-pulse' : ''}`} />
      {children}
    </Badge>
  );
}
