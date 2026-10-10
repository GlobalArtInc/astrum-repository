

import React from 'react';
import classNames from 'classnames';
import { AsList } from '@globalart/astrum-react-shared-components';
import DirectoryListItem from './DirectoryListItem';

import './DirectoryList.scss';

export function DirectoryList({ className, ...props }) {
  return <AsList className={classNames('as-directory-list', className)} {...props} />;
}

DirectoryList.DirectoryListItem = DirectoryListItem;
