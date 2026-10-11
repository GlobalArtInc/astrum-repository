

import React from 'react';
import { Box, Flex, Text, Card, Button } from '@radix-ui/themes';
import { Construction, ArrowLeft, Sparkles, FlaskConical } from 'lucide-react';

import { canPreviewWip, getWipPreviewUrl } from '../config/featureFlags';
import './ComingSoonPage.scss';
import SharedStrings from '../constants/SharedStrings';

/**
 * Coming Soon / Not Implemented Placeholder Page
 * Used for Preview UI features that are still being developed
 * 
 * SECURITY:
 * - Production: Shows Coming Soon only, no way to access WIP
 * - Development: Shows Coming Soon with "View WIP" link for testing
 */
export default function ComingSoonPage({ 
  featureName = SharedStrings.COMING_SOON.THIS_FEATURE,
  featureKey = null,
  description = SharedStrings.COMING_SOON.WERE_WORKING_HARD_BRING_FEATURE,
  showBackButton = true,
  defaultUrl = '#browse/welcome',
}) {
  const handleGoBack = () => {
    window.history.back();
  };

  const handleGoToDefault = () => {
    window.location.hash = defaultUrl.replace('#', '');
  };

  // Check if WIP preview is available (dev mode only)
  const showWipLink = featureKey && canPreviewWip(featureKey);
  const wipUrl = showWipLink ? getWipPreviewUrl(featureKey) : null;

  const handleViewWip = () => {
    if (wipUrl) {
      window.location.href = wipUrl;
    }
  };

  return (
    <div className="coming-soon-page">
      <Card className="coming-soon-page__card">
        <Flex direction="column" align="center" gap="6" p="8">
          {/* Icon */}
          <div className="coming-soon-page__icon">
            <Construction size={64} strokeWidth={1.5} />
            <Sparkles size={24} className="coming-soon-page__sparkle" />
          </div>

          {/* Title */}
          <Text size="8" weight="bold" className="coming-soon-page__title">
            {SharedStrings.COMING_SOON.COMING_SOON}
          </Text>

          {/* Feature Name */}
          <Text size="5" color="gray" className="coming-soon-page__feature">
            {featureName}
          </Text>

          {/* Description */}
          <Text size="3" color="gray" align="center" className="coming-soon-page__description">
            {description}
          </Text>

          {/* Migration Notice */}
          <Box className="coming-soon-page__notice">
            <Text size="2" color="gray">
              {SharedStrings.COMING_SOON.PAGE_PART_ASTRUM_ONE_UI}
            </Text>
          </Box>

          {/* Actions */}
          <Flex gap="3" mt="4">
            {showBackButton && (
              <Button 
                variant="soft" 
                size="3"
                onClick={handleGoBack}
                className="coming-soon-page__button"
              >
                <ArrowLeft size={18} />
                {SharedStrings.COMING_SOON.GO_BACK}
              </Button>
            )}
            <Button 
              variant="solid" 
              size="3"
              onClick={handleGoToDefault}
              className="coming-soon-page__button coming-soon-page__button--primary"
            >
              {SharedStrings.COMING_SOON.GO_DASHBOARD}
            </Button>
          </Flex>

          {/* DEV ONLY: View WIP Version */}
          {showWipLink && (
            <Box className="coming-soon-page__dev-section">
              <Text size="1" color="orange" weight="medium" mb="2">
                🔧 Development Mode
              </Text>
              <Button 
                variant="outline" 
                size="2"
                onClick={handleViewWip}
                className="coming-soon-page__button coming-soon-page__button--wip"
              >
                <FlaskConical size={16} />
                View WIP Version
              </Button>
              <Text size="1" color="gray" mt="2">
                This link is only visible in development.
              </Text>
            </Box>
          )}
        </Flex>
      </Card>
    </div>
  );
}


