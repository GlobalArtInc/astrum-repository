

import { useMemo } from 'react';
import { APIConstants } from '../../../../../../constants/APIConstants';
import { restClient } from '../../../../../../interface/api';
import { useForm, type FormContext } from '../../../../../../interface/form';
import { clearDirtyState, useToast } from '../../../../shared';
import { PREVIEW_UI_SETTINGS_FORM_ID, createPreviewUiSettingsFormMachine, type PreviewUiSettings } from './previewUiSettingsFormMachine';

export function usePreviewUiSettingsForm() {
  const toast = useToast();
  const machine = useMemo(() => createPreviewUiSettingsFormMachine(), []);

  const form = useForm(machine, {
    services: {
      save: async (ctx: FormContext<PreviewUiSettings>) => {
        try {
          await restClient.put(APIConstants.REST.INTERNAL.PREVIEW_UI_SETTINGS, ctx.data);
          clearDirtyState(PREVIEW_UI_SETTINGS_FORM_ID);
          window.location.reload();
        } catch (err) {
          const message = err instanceof Error ? err.message : 'Failed to save Astrum One UI settings';
          toast.error(message);
          throw new Error(message);
        }
      },
    },
  });

  return form;
}
