
import React from 'react';
import { Box } from '@radix-ui/themes';
import { NavItem, NavItemProps } from './NavItem';
import { useRouteVisibility } from './useRouteVisibility';

export interface NavItemBoxProps extends NavItemProps {
  /** Route name to check visibility against (if different from name) */
  visibilityRoute?: string;
}

/**
 * Wraps NavItem in Box only when visible.
 * Prevents empty Box elements from participating in Flex layout
 * and creating extra gaps when NavItem returns null.
 */
export function NavItemBox(props: NavItemBoxProps) {
  const { visibilityRoute, name, href, ...restProps } = props;
  const routeForVisibility = visibilityRoute || name;
  const routeIsVisible = useRouteVisibility(routeForVisibility);

  // For external links (href), always render. For routes, check visibility.
  if (!(routeIsVisible || href)) return null;

  return (
    <Box>
      <NavItem name={name} href={href} {...restProps} />
    </Box>
  );
}
