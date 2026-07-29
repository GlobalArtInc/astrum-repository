

import PropTypes from 'prop-types';
import React, { useEffect, useState } from 'react';
import classnames from 'classnames';
import { NxFontAwesomeIcon, NxGlobalSidebar2NavigationLink } from '@globalart/react-shared-components';
import { faChevronDown, faChevronUp, faLink } from '@fortawesome/free-solid-svg-icons';

import './NavigationLinkWithCollapsibleList.scss';
import CollapsibleListItem from './CollapsibleListItem';
import { useCurrentStateAndParams } from '@uirouter/react';

export function NavigationLinkWithCollapsibleList({ children, text, isSelected, href, icon, name, ...props }) {
  const {
    state: { name: currentPageName }
  } = useCurrentStateAndParams();
  const [isExpanded, setIsExpanded] = useState(currentPageName?.startsWith(name));
  const chevronIcon = isExpanded ? faChevronUp : faChevronDown;

  const onChevronClick = (e) => {
    e.stopPropagation();
    e.preventDefault();
    setIsExpanded(!isExpanded);
  };

  const uniqueId = `collapsible-list-${text}`;

  const itemText = (
    <div className="as-navigation-expandable-link__text">
      {text}
      <button
        aria-label={isExpanded ? 'Collapse Menu' : 'Expand Menu'}
        aria-expanded={isExpanded}
        aria-controls={uniqueId}
        className="as-navigation-expandable-link__chevron"
        onClick={onChevronClick}
      >
        <NxFontAwesomeIcon icon={chevronIcon} />
      </button>
    </div>
  );

  const linkClassnames = classnames('as-navigation-expandable-link__navigation-link', {
    hideIcon: !icon
  });

  const wrapperClasses = classnames('as-navigation-expandable-link', {
    open: isExpanded,
    collapsed: !isExpanded
  });

  useEffect(() => {
    if (currentPageName?.startsWith(name)) {
      setIsExpanded(true);
    }
  }, [currentPageName]);

  return (
    <div className={wrapperClasses}>
      <NxGlobalSidebar2NavigationLink
        className={linkClassnames}
        text={itemText}
        isSelected={isSelected}
        href={href}
        icon={icon ?? faLink}
        {...props}
      />
      <ul
        className="as-navigation-expandable-link__expandable-list"
        aria-hidden={!isExpanded}
        id={uniqueId}
        // Inert is a fairly new standard attribute that prevents the browser from rendering the element and its children
        // This is useful for accessibility, as it prevents screen readers from reading the content when the list is collapsed
        // https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/inert
        inert={!isExpanded}
      >
        {children}
      </ul>
    </div>
  );
}

NavigationLinkWithCollapsibleList.ListItem = CollapsibleListItem;

NavigationLinkWithCollapsibleList.propTypes = {
  children: PropTypes.any,
  href: PropTypes.string.isRequired,
  icon: PropTypes.object,
  isSelected: PropTypes.bool,
  text: PropTypes.oneOfType([PropTypes.string, PropTypes.element]).isRequired,
  name: PropTypes.string.isRequired
};
