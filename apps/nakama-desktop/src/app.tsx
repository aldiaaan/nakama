import './app.css';
import {
  Button,
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from '@nakama/ui/components';

export function App() {
  return (
    <main>
      <Popover>
        <PopoverTrigger render={<Button />}>Click me</PopoverTrigger>
        <PopoverContent>
          <PopoverHeader>
            <PopoverTitle>Popover</PopoverTitle>
            <PopoverDescription>This opens from the button.</PopoverDescription>
          </PopoverHeader>
        </PopoverContent>
      </Popover>
      <p className="text-2xl font-bold text-red-500">adsf</p>
    </main>
  );
}
