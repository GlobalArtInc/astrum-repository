

import React from 'react';
import { useSref, useRouter } from '@uirouter/react';
import { AsList } from '@globalart/astrum-react-shared-components';
import { useIsVisible } from '../../../interface/NavigationUtils';
import classNames from 'classnames';

export default function DirectoryListItem({ text, description, routeName, params, className, ...props }) {
  const routerState = useRouter().stateRegistry.get(routeName);
  const isVisible = useIsVisible(routerState.data.visibilityRequirements);
  const { href } = useSref(routeName, params);

  if (!isVisible) {
    return null;
  }

  return (
    <AsList.LinkItem href={href} className={classNames('as-directory-list-item', className)} {...props}>
      <AsList.Text>{text}</AsList.Text>
      <AsList.Subtext>{description}</AsList.Subtext>
    </AsList.LinkItem>
  );
}
