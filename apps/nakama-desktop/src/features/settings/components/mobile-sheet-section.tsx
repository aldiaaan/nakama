import { AnimatedContainer, Button, MobileSheet, PreferenceCard } from '@nakama/ui/components';
import { useQuery } from '@tanstack/react-query';

const lorem =
  'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.';

const routes = [
  { id: 'home', title: 'Home', element: <HomeScreen /> },
  { id: 'detail', title: 'Detail', element: <DetailScreen /> },
];

export function MobileSheetSection() {
  return (
    <PreferenceCard.Root>
      <PreferenceCard.Header>
        <PreferenceCard.Title>Mobile sheet</PreferenceCard.Title>
        <PreferenceCard.Description>
          A bottom sheet with its own stack, separate from the app router.
        </PreferenceCard.Description>
      </PreferenceCard.Header>
      <PreferenceCard.Content>
        <MobileSheet.Root initialId="home" routes={routes}>
          <MobileSheet.Trigger render={<Button />}>Open sheet</MobileSheet.Trigger>
          <MobileSheet.Content description="A sample stack with two screens." title="Mobile sheet" />
        </MobileSheet.Root>
      </PreferenceCard.Content>
    </PreferenceCard.Root>
  );
}

function HomeScreen() {
  const copy = useQuery({
    queryKey: ['settings', 'mobile-sheet-copy'],
    queryFn: async () => {
      await new Promise((resolve) => {
        window.setTimeout(resolve, 900);
      });
      return lorem;
    },
    gcTime: 0,
  });

  return (
    <div className="flex flex-col gap-4">
      <AnimatedContainer className="w-full">
        {copy.isPending ? (
          <div className="flex h-10 w-full items-center justify-center" role="status">
            <span className="border-muted-foreground/30 border-t-foreground size-6 shrink-0 animate-spin rounded-full border-2" />
            <span className="sr-only">Loading</span>
          </div>
        ) : (
          <p className="text-muted-foreground w-full">{copy.data}</p>
        )}
      </AnimatedContainer>
      <MobileSheet.Link className="w-full" to="detail">
        Continue
      </MobileSheet.Link>
    </div>
  );
}

function DetailScreen() {
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-2">
        <p className="text-muted-foreground">
          This screen is taller, so the sheet eases to the new height instead of jumping.
        </p>
        <p className="text-muted-foreground">
          Back pops this frame. The close button dismisses the sheet and the next open starts at home again.
        </p>
      </div>
      <MobileSheet.Back className="w-full" variant="outline">
        Back
      </MobileSheet.Back>
    </div>
  );
}
