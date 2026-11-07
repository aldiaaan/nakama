'use client';

import { Progress as ProgressPrimitive } from '@base-ui/react/progress';
import { cn } from 'cn';
import { motion, useReducedMotion } from 'motion/react';

function Progress({ className, children, value, ...props }: ProgressPrimitive.Root.Props) {
  return (
    <ProgressPrimitive.Root
      value={value}
      data-slot="progress"
      className={cn('flex flex-wrap gap-3', className)}
      {...props}
    >
      {children}
      <ProgressTrack>
        <ProgressIndicator />
      </ProgressTrack>
    </ProgressPrimitive.Root>
  );
}

function ProgressTrack({ className, ...props }: ProgressPrimitive.Track.Props) {
  return (
    <ProgressPrimitive.Track
      className={cn('relative flex h-3 w-full items-center overflow-x-hidden rounded-4xl bg-muted', className)}
      data-slot="progress-track"
      {...props}
    />
  );
}

function ProgressIndicator({ className, render, ...props }: ProgressPrimitive.Indicator.Props) {
  const reduceMotion = useReducedMotion();
  return (
    <ProgressPrimitive.Indicator
      data-slot="progress-indicator"
      className={cn('h-full bg-primary', className)}
      render={
        render ??
        ((elementProps) => {
          // Motion redefines these DOM handlers, so they are not forwarded.
          const {
            style,
            onDrag: _onDrag,
            onDragStart: _onDragStart,
            onDragEnd: _onDragEnd,
            onAnimationStart: _onAnimationStart,
            onAnimationEnd: _onAnimationEnd,
            ...elementRest
          } = elementProps;
          if (style?.width == null) {
            return <motion.div {...elementRest} style={style} />;
          }
          const { width, ...restStyle } = style;
          return (
            <motion.div
              {...elementRest}
              style={restStyle}
              initial={{ width: '0%' }}
              animate={{ width }}
              transition={reduceMotion ? { duration: 0 } : { duration: 0.7, ease: [0.22, 1, 0.36, 1] }}
            />
          );
        })
      }
      {...props}
    />
  );
}

function ProgressLabel({ className, ...props }: ProgressPrimitive.Label.Props) {
  return (
    <ProgressPrimitive.Label className={cn('text-sm font-medium', className)} data-slot="progress-label" {...props} />
  );
}

function ProgressValue({ className, ...props }: ProgressPrimitive.Value.Props) {
  return (
    <ProgressPrimitive.Value
      className={cn('ml-auto text-sm text-muted-foreground tabular-nums', className)}
      data-slot="progress-value"
      {...props}
    />
  );
}

export { Progress, ProgressTrack, ProgressIndicator, ProgressLabel, ProgressValue };
