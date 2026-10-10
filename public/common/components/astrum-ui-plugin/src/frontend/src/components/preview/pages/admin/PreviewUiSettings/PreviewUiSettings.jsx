
import React from 'react';
import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle,
  Section,
} from '../../../../layout';
import { ExtJS } from '../../../../../interface/ExtJS';
import { FormUtils } from '../../../../../interface/FormUtils';
import {
  AsLoadWrapper,
  AsCheckbox,
  AsButton,
  AsFieldset,
  AsStatefulForm,
  AsTooltip,
} from '@globalart/react-shared-components';

import UIStrings from '../../../constants/UIStrings';
import { usePreviewUiSettingsForm } from '../../settings/system/preview-ui/usePreviewUiSettingsForm';

export default function PreviewUiSettings() {
  const canEdit = ExtJS.checkPermission('astrum:settings:update');
  const form = usePreviewUiSettingsForm();

  const anonymousCb = form.checkbox('anonymousEnabled');
  const loggedInCb = form.checkbox('loggedInEnabled');
  const defaultToPreviewCb = form.checkbox('defaultToPreviewUi');
  const disableSwitchFeedbackCb = form.checkbox('disableSwitchFeedback');

  return (
    <Page>
      <PageHeader>
        <PageTitle
          icon={UIStrings.PREVIEW_UI_SETTINGS.MENU.icon}
          text={UIStrings.PREVIEW_UI_SETTINGS.MENU.text}
          description={UIStrings.PREVIEW_UI_SETTINGS.description}
        />
      </PageHeader>
      <ContentBody className="as-preview-ui-settings">
        <Section>
          <AsLoadWrapper
            loading={form.isLoading}
            error={form.loadError}
            retryHandler={() => form.send({ type: 'RETRY' })}
          >
            <AsStatefulForm
              onSubmit={() => form.submit()}
              loading={form.isSaving}
              submitError={form.saveError ? String(form.saveError) : null}
              validationErrors={form.hasValidationErrors ? Object.values(form.validationErrors).find(Boolean) : null}
              additionalFooterBtns={
                <AsTooltip title={FormUtils.discardTooltip({ isPristine: form.isPristine })}>
                  <AsButton
                    type="button"
                    className={form.isPristine ? 'disabled' : undefined}
                    onClick={() => form.reset()}
                  >
                    {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
                  </AsButton>
                </AsTooltip>
              }
            >
              {() => <>
                {/* ASTRUM-52024: Disable Classic UI checkbox hidden until Astrum One UI reaches feature parity */}

                <AsFieldset label={UIStrings.PREVIEW_UI_SETTINGS.ANONYMOUS.label}>
                  <AsCheckbox
                    checkboxId="anonymousEnabled"
                    isChecked={anonymousCb.checked}
                    onChange={() => anonymousCb.onChange(!anonymousCb.checked)}
                    disabled={!canEdit}
                  >
                    {UIStrings.PREVIEW_UI_SETTINGS.ANONYMOUS.description}
                  </AsCheckbox>
                </AsFieldset>

                <AsFieldset label={UIStrings.PREVIEW_UI_SETTINGS.LOGGEDIN.label}>
                  <AsCheckbox
                    checkboxId="loggedInEnabled"
                    isChecked={loggedInCb.checked}
                    onChange={() => loggedInCb.onChange(!loggedInCb.checked)}
                    disabled={!canEdit}
                  >
                    {UIStrings.PREVIEW_UI_SETTINGS.LOGGEDIN.description}
                  </AsCheckbox>
                </AsFieldset>

                <AsFieldset label={UIStrings.PREVIEW_UI_SETTINGS.ROLLOUT.title}>
                  <AsCheckbox
                    checkboxId="defaultToPreviewUi"
                    isChecked={defaultToPreviewCb.checked}
                    onChange={() => defaultToPreviewCb.onChange(!defaultToPreviewCb.checked)}
                    disabled={!canEdit}
                  >
                    {UIStrings.PREVIEW_UI_SETTINGS.ROLLOUT.DEFAULT_TO_PREVIEW.helpText}
                  </AsCheckbox>
                  <AsCheckbox
                    checkboxId="disableSwitchFeedback"
                    isChecked={disableSwitchFeedbackCb.checked}
                    onChange={() => disableSwitchFeedbackCb.onChange(!disableSwitchFeedbackCb.checked)}
                    disabled={!canEdit}
                  >
                    {UIStrings.PREVIEW_UI_SETTINGS.ROLLOUT.DISABLE_SWITCH_FEEDBACK.helpText}
                  </AsCheckbox>
                </AsFieldset>
              </>}
            </AsStatefulForm>
          </AsLoadWrapper>
        </Section>
      </ContentBody>
    </Page>
  );
}
