

import { assign } from 'xstate';
import { createFormMachine } from '../../../../../../interface/form';
import { restClient } from '../../../../../../interface/api';
import { APIConstants } from '../../../../../../constants/APIConstants';
import type { FormContext, ValidationErrors } from '../../../../../../interface/form';

export interface PreviewUiSettings {
  anonymousEnabled: boolean;
  loggedInEnabled: boolean;
  defaultToPreviewUi: boolean;
  disableLegacyUi: boolean;
  disableSwitchFeedback: boolean;
}

export const PREVIEW_UI_SETTINGS_FORM_ID = 'preview-ui-settings-form';

export const DEFAULT_PREVIEW_UI_SETTINGS: PreviewUiSettings = {
  anonymousEnabled: false,
  loggedInEnabled: false,
  defaultToPreviewUi: false,
  disableLegacyUi: false,
  disableSwitchFeedback: false,
};

export const LOCKOUT_ERROR_MESSAGE =
  'Cannot disable Classic UI when both Anonymous and Logged-in access to Astrum One UI are disabled. ' +
  'At least one UI access method must remain enabled to prevent lockout.';

function validatePreviewUiSettings(data: PreviewUiSettings): ValidationErrors {
  const errors: ValidationErrors = {};
  if (data.disableLegacyUi && !data.anonymousEnabled && !data.loggedInEnabled) {
    errors.disableLegacyUi = LOCKOUT_ERROR_MESSAGE;
  }
  return errors;
}

export function createPreviewUiSettingsFormMachine() {
  return createFormMachine<PreviewUiSettings>({
    id: PREVIEW_UI_SETTINGS_FORM_ID,
    context: {
      data: { ...DEFAULT_PREVIEW_UI_SETTINGS },
    },
    actions: {
      // createFormMachine invokes the 'validate' named action on UPDATE events and after load.
      // This contract is verified by previewUiSettingsFormMachine.test.ts (shouldReportErrorWhenDisableLegacyAndBothAccessDisabled).
      validate: assign((ctx: FormContext<PreviewUiSettings>) => ({
        validationErrors: validatePreviewUiSettings(ctx.data),
      })),
    },
    services: {
      load: async () => {
        const data = await restClient.get<PreviewUiSettings>(APIConstants.REST.INTERNAL.PREVIEW_UI_SETTINGS);
        return { data };
      },
      save: async (ctx) => {
        await restClient.put(APIConstants.REST.INTERNAL.PREVIEW_UI_SETTINGS, ctx.data);
      },
    },
  });
}
