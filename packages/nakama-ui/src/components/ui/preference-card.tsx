'use client';

import { cn } from 'cn';
import type { ComponentProps } from 'react';

function PreferenceCardRoot({ className, ...props }: ComponentProps<'section'>) {
  return (
    <section
      data-slot="preference-card"
      className={cn('flex flex-col gap-4 rounded-2xl bg-card p-6 text-sm ring-1 ring-foreground/10', className)}
      {...props}
    />
  );
}

function PreferenceCardHeader({ className, ...props }: ComponentProps<'div'>) {
  return <div data-slot="preference-card-header" className={cn('flex flex-col gap-1', className)} {...props} />;
}

function PreferenceCardTitle({ className, ...props }: ComponentProps<'h2'>) {
  return <h2 data-slot="preference-card-title" className={cn('text-base font-medium', className)} {...props} />;
}

function PreferenceCardDescription({ className, ...props }: ComponentProps<'p'>) {
  return <p data-slot="preference-card-description" className={cn('text-muted-foreground', className)} {...props} />;
}

function PreferenceCardContent({ className, ...props }: ComponentProps<'div'>) {
  return <div data-slot="preference-card-content" className={cn(className)} {...props} />;
}

function PreferenceCardFooter({ className, ...props }: ComponentProps<'div'>) {
  return <div data-slot="preference-card-footer" className={cn('flex flex-col gap-2', className)} {...props} />;
}

function PreferenceCardError({ className, ...props }: ComponentProps<'p'>) {
  return (
    <p
      data-slot="preference-card-error"
      className={cn('font-sans font-medium text-destructive', className)}
      {...props}
    />
  );
}

export const PreferenceCard = {
  Root: PreferenceCardRoot,
  Header: PreferenceCardHeader,
  Title: PreferenceCardTitle,
  Description: PreferenceCardDescription,
  Content: PreferenceCardContent,
  Footer: PreferenceCardFooter,
  Error: PreferenceCardError,
};
