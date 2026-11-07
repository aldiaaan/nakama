import { themeChoices, useSaveSettings, type SettingsTheme } from '@nakama/api';
import { RadioGroup, RadioGroupItem } from '@nakama/ui/components';
import { useRef } from 'react';
import { Controller, useForm } from 'react-hook-form';

import { useSettings } from '../hooks/use-settings';
import { useSettingsApi } from '../hooks/use-settings-api';

type AppearanceValues = {
  theme: SettingsTheme;
};

export function AppearanceForm() {
  const theme = useSettings((state) => state.theme);
  const path = useSettings((state) => state.path);
  const store = useSettingsApi();
  const save = useSaveSettings();
  const request = useRef(0);
  const form = useForm<AppearanceValues>({ values: { theme } });

  function selectTheme(next: SettingsTheme) {
    if (next === store.getState().theme) {
      return;
    }
    const previous = store.getState().theme;
    const id = request.current + 1;
    request.current = id;
    store.setState({ theme: next });
    save.mutate(next, {
      onSuccess: (saved) => {
        if (id !== request.current) {
          return;
        }
        store.setState({ theme: saved.theme, path: saved.path });
      },
      onError: () => {
        if (id !== request.current) {
          return;
        }
        store.setState({ theme: previous });
      },
    });
  }

  return (
    <form className="flex flex-col gap-4">
      <Controller
        control={form.control}
        name="theme"
        render={({ field }) => (
          <RadioGroup
            value={field.value}
            disabled={save.isPending}
            onValueChange={(value) => {
              field.onChange(value);
              selectTheme(value);
            }}
          >
            {themeChoices.map((choice) => {
              const labelId = `appearance-theme-${choice.value}`;
              return (
                <div key={choice.value} className="bg-muted/60 flex items-start gap-3 rounded-xl px-3 py-3">
                  <RadioGroupItem value={choice.value} aria-labelledby={labelId} className="mt-0.5" />
                  <span>
                    <span id={labelId} className="font-medium">
                      {choice.label}
                    </span>
                    <span className="text-muted-foreground block">{choice.description}</span>
                  </span>
                </div>
              );
            })}
          </RadioGroup>
        )}
      />
      {path ? <p className="text-muted-foreground">{path}</p> : null}
      {save.statusLabel ? <p className="text-muted-foreground">{save.statusLabel}</p> : null}
      {save.errorMessage ? <p className="text-destructive">{save.errorMessage}</p> : null}
    </form>
  );
}
