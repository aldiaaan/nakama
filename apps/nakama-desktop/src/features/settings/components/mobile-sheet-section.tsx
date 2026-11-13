import { Button, MobileSheet, PreferenceCard } from '@nakama/ui/components';

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
  return (
    <div className="flex flex-col gap-4">
      <p className="text-muted-foreground">A short screen in this sheet's own stack.</p>
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
