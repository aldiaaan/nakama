import { themeChoices, useLoadSettings, useSaveSettings, type SettingsTheme } from '@nakama/api';
import { PreferenceCard, PreferenceTile } from '@nakama/ui/components';
import { useRef } from 'react';

import { useSettings } from '../hooks/use-settings';
import { useSettingsApi } from '../hooks/use-settings-api';

export function AppearancePreferenceSection() {
  const theme = useSettings((state) => state.theme);
  const path = useSettings((state) => state.path);
  const store = useSettingsApi();
  const loaded = useLoadSettings();
  const save = useSaveSettings();
  const request = useRef(0);

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
    <PreferenceCard.Root>
      <PreferenceCard.Header>
        <PreferenceCard.Title>Appearance</PreferenceCard.Title>
        <PreferenceCard.Description>Saved to config.ini in the app config folder.</PreferenceCard.Description>
      </PreferenceCard.Header>
      <PreferenceCard.Content>
        {loaded.statusLabel ? (
          <p className="text-muted-foreground">{loaded.statusLabel}</p>
        ) : (
          <PreferenceTile.Group value={theme} disabled={save.isPending} onValueChange={selectTheme}>
            {themeChoices.map((choice) => (
              <PreferenceTile.Item key={choice.value} value={choice.value}>
                <PreferenceTile.Indicator />
                <PreferenceTile.Content>
                  <PreferenceTile.Title>{choice.label}</PreferenceTile.Title>
                  <PreferenceTile.Description>{choice.description}</PreferenceTile.Description>
                </PreferenceTile.Content>
              </PreferenceTile.Item>
            ))}
          </PreferenceTile.Group>
        )}
      </PreferenceCard.Content>
      <PreferenceCard.Footer>
        {/* {path ? <p className="truncate select-all">{path}</p> : null} */}
        {save.statusLabel ? <p className="text-muted-foreground">{save.statusLabel}</p> : null}
        {loaded.errorMessage ? <PreferenceCard.Error>{loaded.errorMessage}</PreferenceCard.Error> : null}
        {save.errorMessage ? <PreferenceCard.Error>{save.errorMessage}</PreferenceCard.Error> : null}
      </PreferenceCard.Footer>
    </PreferenceCard.Root>
  );
}
