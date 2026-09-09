



import React from 'react';
import { Box, Text, Callout, Link } from '@radix-ui/themes';
import { ExternalLink, FileArchive, Info } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { SettingsFormSection, SettingsButton, SettingsAlert } from '../../../../shared/form';
import { PageHeader } from '../../../../shared';

import './SupportRequestPage.scss';

interface SupportRequestPageProps {
  className?: string;
}

// External support request URL (Pro Edition)
const SUPPORT_REQUEST_URL = 'https://links.globalart.dev/products/astrum/pro/support-request';

// Internal Preview UI route to Support ZIP
const SUPPORT_ZIP_ROUTE = '#preview/admin/support/supportzip';

/**
 * SupportRequestPage - Support Request page for Preview UI
 *
 * Provides information about submitting support requests to GlobalArt.
 * This is a Pro Edition feature only.
 *
 * Permission: astrum:atlas:create
 * Route: #preview/admin/support/supportrequest
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
          title="Support Request"
          description="Submit a support request to GlobalArt"
          breadcrumbs={[
            { label: 'Settings', onClick: () => { window.location.hash = '#preview/admin/settings'; } },
            { label: 'Support Request' },
          ]}
        />

        <SettingsAlert type="warning" data-testid="support-request-pro-only-warning">
          Support Request is only available in Astrum Repository Pro Edition.
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
          title="Support Request"
          description="Submit a support request to GlobalArt"
          breadcrumbs={[
            { label: 'Settings', onClick: () => { window.location.hash = '#preview/admin/settings'; } },
            { label: 'Support Request' },
          ]}
        />

        <SettingsAlert type="warning" data-testid="support-request-permission-warning">
          You do not have permission to submit support requests. Contact an administrator.
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
        title="Support Request"
        description="Submit a support request to GlobalArt"
        breadcrumbs={[
          { label: 'Settings', onClick: () => { window.location.hash = '#preview/admin/settings'; } },
          { label: 'Support Request' },
        ]}
      />

      {/* Main Content Section */}
      <SettingsFormSection
        title="Submit a Support Request"
        description="Get help from GlobalArt's expert support team"
      >
        {/* Instructions */}
        <Box className="support-request-page__content">
          <Text as="p" size="2" className="support-request-page__text">
            Please include a complete description of your problem and steps to allow us to reproduce
            the problem (if available).
          </Text>

          {/* Support ZIP recommendation */}
          <Box className="support-request-page__callout">
            <Callout.Root color="blue" variant="soft">
              <Callout.Icon>
                <Info size={16} />
              </Callout.Icon>
              <Callout.Text>
                Attaching a support ZIP to your request will help our engineers give you a faster response.
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
                Generate a Support ZIP
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
              Submit Support Request
            </SettingsButton>
          </Box>
        </Box>
      </SettingsFormSection>
    </Box>
  );
}

export default SupportRequestPage;

