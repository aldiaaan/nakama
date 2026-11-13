import { AnimatedContainer, Button, PreferenceCard } from '@nakama/ui/components';
import { useState } from 'react';

export function AnimatedContainerSection() {
  const [expanded, setExpanded] = useState(false);

  return (
    <PreferenceCard.Root>
      <PreferenceCard.Header>
        <PreferenceCard.Title>Animated container</PreferenceCard.Title>
        <PreferenceCard.Description>The box eases to the new size when its content changes.</PreferenceCard.Description>
      </PreferenceCard.Header>
      <PreferenceCard.Content className="flex flex-col items-start gap-4">
        <AnimatedContainer className="bg-muted px-4 py-3" style={{ borderRadius: 16 }}>
          {expanded ? (
            <div className="flex w-72 flex-col gap-2">
              <p className="text-muted-foreground">
                This block is wider and taller, so the container eases to the new size.
              </p>
              <p className="text-muted-foreground">Show less eases it back to the short line.</p>
            </div>
          ) : (
            <p className="text-muted-foreground">A short line.</p>
          )}
        </AnimatedContainer>
        <p className="flex items-center gap-2">
          <span className="text-muted-foreground">Label</span>
          <AnimatedContainer as="span" className="bg-muted px-2 py-0.5" style={{ borderRadius: 999 }}>
            {expanded ? 'Expanded label' : 'Short'}
          </AnimatedContainer>
        </p>
        <Button type="button" variant="outline" onClick={() => setExpanded((current) => !current)}>
          {expanded ? 'Show less' : 'Show more'}
        </Button>
      </PreferenceCard.Content>
    </PreferenceCard.Root>
  );
}
