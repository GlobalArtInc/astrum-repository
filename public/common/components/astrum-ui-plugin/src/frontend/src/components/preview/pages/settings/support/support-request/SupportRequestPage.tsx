



import React from 'react';
import { Box, Text, Callout, Link } from '@radix-ui/themes';
import { ExternalLink, FileArchive, Info } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { SettingsFormSection, SettingsButton, SettingsAlert } from '../../../../shared/form';
import { PageHeader } from '../../../../shared';

import './SupportRequestPage.scss';
import SupportRequestStrings from '../../../../constants/pages/settings/support/SupportRequestStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface SupportRequestPageProps {
  className?: string;
}

// External support request URL (Pro Edition)
const SUPPORT_REQUEST_URL = 'https://links.globalart.dev/products/astrum/pro/support-request';

// Internal Preview UI route to Support ZIP
const SUPPORT_ZIP_ROUTE = '#admin/support/supportzip';

/**
 * SupportRequestPage - Support Request page for Preview UI
 *
 * Provides information about submitting support requests to GlobalArt.
 * This is a Pro Edition feature only.
 *
 * Permission: astrum:atlas:create
 * Route: #admin/support/supportrequest
 */
export function SupportRequestPage({ className }: SupportRequestPageProps) {
  // Check permission - astrum:atlas:create required
  const canCreate = ExtJS.checkPermission('astrum:atlas:create');

  // Check if Pro Edition - use the proper helper method
  const isPro = ExtJS.isProEdition();

  // Handle external link click
  const handleSubmitRequest = () => {
    window.open(SUPPORT_REQUEST_URL, '_blank', 'noopener,noreferrer');
  };

  // Handle internal navigation to Support ZIP
  const handleSupportZipClick = () => {
    window.location.hash = SUPPORT_ZIP_ROUTE.replace('#', '');
  };

  // Show warning if not Pro Edition
  if (!isPro) {
    return (
      <Box
        className={`support-request-page ${className || ''}`.trim()}
        data-testid="support-request-page"
        data-edition="oss"
      >
        {/* Header */}
        <PageHeader
          title={SupportRequestStrings.PAGE.SUPPORT_REQUEST}
          description={SupportRequestStrings.PAGE.SUBMIT_SUPPORT_REQUEST_GLOBALART}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => { window.location.hash = '#admin/settings'; } },
            { label: SupportRequestStrings.PAGE.SUPPORT_REQUEST },
          ]}
        />

        <SettingsAlert type="warning" data-testid="support-request-pro-only-warning">
          {SupportRequestStrings.PAGE.SUPPORT_REQUEST_ONLY_AVAILABLE_ASTRUM}
        </SettingsAlert>
      </Box>
    );
  }

  // Show warning if user lacks permission
  if (!canCreate) {
    return (
      <Box
        className={`support-request-page ${className || ''}`.trim()}
        data-testid="support-request-page"
        data-permission="denied"
      >
        {/* Header */}
        <PageHeader
          title={SupportRequestStrings.PAGE.SUPPORT_REQUEST}
          description={SupportRequestStrings.PAGE.SUBMIT_SUPPORT_REQUEST_GLOBALART}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => { window.location.hash = '#admin/settings'; } },
            { label: SupportRequestStrings.PAGE.SUPPORT_REQUEST },
          ]}
        />

        <SettingsAlert type="warning" data-testid="support-request-permission-warning">
          {SupportRequestStrings.PAGE.YOU_DO_NOT_HAVE_PERMISSION}
        </SettingsAlert>
      </Box>
    );
  }

  return (
    <Box
      className={`support-request-page ${className || ''}`.trim()}
      data-testid="support-request-page"
      data-edition="pro"
    >
      {/* Header */}
      <PageHeader
        title={SupportRequestStrings.PAGE.SUPPORT_REQUEST}
        description={SupportRequestStrings.PAGE.SUBMIT_SUPPORT_REQUEST_GLOBALART}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => { window.location.hash = '#admin/settings'; } },
          { label: SupportRequestStrings.PAGE.SUPPORT_REQUEST },
        ]}
      />

      {/* Main Content Section */}
      <SettingsFormSection
        title={SupportRequestStrings.PAGE.SUBMIT_SUPPORT_REQUEST}
        description={SupportRequestStrings.PAGE.GET_HELP_GLOBALARTS_EXPERT_SUPPORT}
      >
        {/* Instructions */}
        <Box className="support-request-page__content">
          <Text as="p" size="2" className="support-request-page__text">
            {SupportRequestStrings.PAGE.PLEASE_INCLUDE_COMPLETE_DESCRIPTION_PROBLEM}
          </Text>

          {/* Support ZIP recommendation */}
          <Box className="support-request-page__callout">
            <Callout.Root color="blue" variant="soft">
              <Callout.Icon>
                <Info size={16} />
              </Callout.Icon>
              <Callout.Text>
                {SupportRequestStrings.PAGE.ATTACHING_SUPPORT_ZIP_REQUEST_HELP}
              </Callout.Text>
            </Callout.Root>
            <Box mt="2">
              <Link
                href={SUPPORT_ZIP_ROUTE}
                onClick={(e) => {
                  e.preventDefault();
                  handleSupportZipClick();
                }}
                data-testid="support-request-supportzip-link"
                className="support-request-page__link"
              >
                <FileArchive size={14} style={{ marginRight: '4px', verticalAlign: 'middle' }} />
                {SupportRequestStrings.PAGE.GENERATE_SUPPORT_ZIP}
              </Link>
            </Box>
          </Box>

          {/* Submit Button */}
          <Box className="support-request-page__actions" mt="4">
            <SettingsButton
              variant="primary"
              onClick={handleSubmitRequest}
              icon={ExternalLink}
              data-testid="support-request-submit-button"
            >
              {SupportRequestStrings.PAGE.SUBMIT_SUPPORT_REQUEST_2}
            </SettingsButton>
          </Box>
        </Box>
      </SettingsFormSection>
    </Box>
  );
}

export default SupportRequestPage;

