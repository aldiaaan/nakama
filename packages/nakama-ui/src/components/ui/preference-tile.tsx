'use client';

import { Radio as RadioPrimitive } from '@base-ui/react/radio';
import { RadioGroup as RadioGroupPrimitive } from '@base-ui/react/radio-group';
import { cn } from 'cn';
import { motion, useReducedMotion } from 'motion/react';
import type { ComponentProps } from 'react';

function PreferenceTileGroup<Value>({ className, ...props }: RadioGroupPrimitive.Props<Value>) {
  return (
    <RadioGroupPrimitive data-slot="preference-tile-group" className={cn('grid w-full gap-3', className)} {...props} />
  );
}

function PreferenceTileItem({ className, render, ...props }: RadioPrimitive.Root.Props) {
  const reduceMotion = useReducedMotion();

  return (
    <RadioPrimitive.Root
      data-slot="preference-tile-item"
      className={cn(
        'group/preference-tile flex w-full cursor-pointer items-start gap-3 rounded-xl bg-muted/60 px-3 py-3 text-left outline-none select-none focus-visible:ring-3 focus-visible:ring-ring/50 data-disabled:pointer-events-none data-disabled:opacity-50',
        className
      )}
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
          return (
            <motion.span
              {...elementRest}
              style={style}
              whileTap={reduceMotion ? undefined : { scale: 0.992 }}
              transition={reduceMotion ? { duration: 0 } : { type: 'spring', duration: 0.22, bounce: 0.2 }}
            />
          );
        })
      }
      {...props}
    />
  );
}

function PreferenceTileIndicator({ className, ...props }: ComponentProps<'span'>) {
  return (
    <span
      aria-hidden
      data-slot="preference-tile-indicator"
      className={cn(
        'mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border border-input group-data-checked/preference-tile:border-primary group-data-checked/preference-tile:bg-primary',
        className
      )}
      {...props}
    >
      <RadioPrimitive.Indicator data-slot="preference-tile-indicator-dot" className="flex items-center justify-center">
        <span className="bg-primary-foreground size-2 rounded-full" />
      </RadioPrimitive.Indicator>
    </span>
  );
}

function PreferenceTileContent({ className, ...props }: ComponentProps<'span'>) {
  return <span data-slot="preference-tile-content" className={cn('min-w-0 flex-1', className)} {...props} />;
}

function PreferenceTileTitle({ className, ...props }: ComponentProps<'span'>) {
  return <span data-slot="preference-tile-title" className={cn('block font-medium', className)} {...props} />;
}

function PreferenceTileDescription({ className, ...props }: ComponentProps<'span'>) {
  return (
    <span data-slot="preference-tile-description" className={cn('block text-muted-foreground', className)} {...props} />
  );
}

export const PreferenceTile = {
  Group: PreferenceTileGroup,
  Item: PreferenceTileItem,
  Indicator: PreferenceTileIndicator,
  Content: PreferenceTileContent,
  Title: PreferenceTileTitle,
  Description: PreferenceTileDescription,
};
