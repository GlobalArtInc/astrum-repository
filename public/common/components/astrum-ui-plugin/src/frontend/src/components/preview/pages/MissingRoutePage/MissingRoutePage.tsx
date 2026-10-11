

import React from 'react';
import { Flex, Text, Heading, Button, Box } from '@radix-ui/themes';
import { Ghost, BookOpen, Home } from 'lucide-react';
import { useSref } from '@uirouter/react';
import { ROUTE_NAMES } from '../../constants/RouteNames';

import './MissingRoutePage.scss';
import AuditStrings from '../../constants/pages/audit/AuditStrings';

/**
 * Preview UI 404 Page - Missing Route
 *
 * A modern, Radix UI-based 404 page for the Preview UI.
 * Features:
 * - Clean, centered layout with ghost icon
 * - Clear messaging about the missing resource
 * - Actions to return home or visit documentation
 * - Dark mode support
 */
export function MissingRoutePage(): JSX.Element {
  const { href: dashboardHref } = useSref(ROUTE_NAMES.BROWSE.WELCOME.ROOT);

  return (
    <Flex
      direction="column"
      align="center"
      justify="center"
      className="as-missing-route-page"
    >
      <Box className="as-missing-route-page__content">
        {/* Ghost Icon */}
        <Box className="as-missing-route-page__ghost-icon">
          <Ghost size={80} strokeWidth={1.5} />
        </Box>

        {/* 404 Code */}
        <Text className="as-missing-route-page__code" as="div">
          404
        </Text>

        {/* Title */}
        <Heading size="6" className="as-missing-route-page__title">
          {AuditStrings.MISSING_ROUTE.RESOURCE_NOT_FOUND}
        </Heading>

        {/* Divider */}
        <Box className="as-missing-route-page__divider" />

        {/* Message */}
        <Text size="3" color="gray" className="as-missing-route-page__message">
          {AuditStrings.MISSING_ROUTE.RESOURCE_NOT_AVAILABLE_MAY_HAVE}
        </Text>

        {/* Actions */}
        <Flex direction="column" gap="4" align="center" className="as-missing-route-page__actions">
          <Text size="2" color="gray" className="as-missing-route-page__actions-message">
            {AuditStrings.MISSING_ROUTE.YOU_MIGHT_WANT}
          </Text>

          <Flex gap="3" wrap="wrap" justify="center" className="as-missing-route-page__buttons">
            <Button asChild variant="solid" size="3">
              <a href={dashboardHref} data-analytics-id="as-404-page-dashboard-lnk">
                <Home size={16} />
                {AuditStrings.MISSING_ROUTE.RETURN_DASHBOARD}
              </a>
            </Button>

            <Button asChild variant="soft" size="3">
              <a
                href="https://links.globalart.dev/products/astrum/docs"
                target="_blank"
                rel="noopener noreferrer"
                data-analytics-id="as-404-page-support-lnk"
              >
                <BookOpen size={16} />
                {AuditStrings.MISSING_ROUTE.VISIT_DOCUMENTATION}
              </a>
            </Button>
          </Flex>
        </Flex>
      </Box>
    </Flex>
  );
}

export default MissingRoutePage;
