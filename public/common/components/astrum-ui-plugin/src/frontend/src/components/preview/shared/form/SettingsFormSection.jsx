

import React from 'react';
import PropTypes from 'prop-types';
import { Box, Flex, Heading, Text, Separator } from '@radix-ui/themes';

import './SettingsFormSection.scss';

/**
 * SettingsFormSection - Groups related form fields with a title
 * 
 * @example
 * <SettingsFormSection 
 *   title="S3 Settings" 
 *   description="Configure S3 bucket settings"
 *   icon={<Cloud size={20} />}
 * >
 *   <SettingsTextInput name="bucket" ... />
 *   <SettingsTextInput name="region" ... />
 * </SettingsFormSection>
 */
export function SettingsFormSection({
  title,
  description,
  children,
  icon,
  collapsible = false,
  defaultCollapsed = false,
  className = '',
}) {
  const [collapsed, setCollapsed] = React.useState(defaultCollapsed);

  const handleToggle = () => {
    if (collapsible) {
      setCollapsed(!collapsed);
    }
  };

  return (
    <Box className={`settings-section ${collapsed ? 'settings-section--collapsed' : ''} ${className}`.trim()}>
      {title && (
        <Box
          className={`settings-section__header ${collapsible ? 'settings-section__header--collapsible' : ''}`}
          onClick={handleToggle}
          role={collapsible ? 'button' : undefined}
          tabIndex={collapsible ? 0 : undefined}
          onKeyDown={collapsible ? (e) => e.key === 'Enter' && handleToggle() : undefined}
          aria-expanded={collapsible ? !collapsed : undefined}
        >
          <Flex align="center" gap="2">
            {icon && (
              <Box className="settings-section__icon">
                {icon}
              </Box>
            )}
            <Box>
              <Heading as="h2" size="3" weight="medium" className="settings-section__title">
                {collapsible && (
                  <span className={`settings-section__chevron ${collapsed ? 'settings-section__chevron--collapsed' : ''}`}>
                    ▾
                  </span>
                )}
                {title}
              </Heading>
              {description && (
                <Text as="p" size="2" className="settings-section__description">
                  {description}
                </Text>
              )}
            </Box>
          </Flex>
        </Box>
      )}
      {(!collapsible || !collapsed) && (
        <Box className="settings-section__content">
          {children}
        </Box>
      )}
      <Separator size="4" className="settings-section__separator" />
    </Box>
  );
}

SettingsFormSection.propTypes = {
  /** Section title */
  title: PropTypes.string,
  /** Optional description text */
  description: PropTypes.string,
  /** Form fields */
  children: PropTypes.node.isRequired,
  /** Optional icon element to display before the title */
  icon: PropTypes.node,
  /** Whether section can be collapsed */
  collapsible: PropTypes.bool,
  /** Initial collapsed state */
  defaultCollapsed: PropTypes.bool,
  /** Additional CSS class */
  className: PropTypes.string,
};

export default SettingsFormSection;
