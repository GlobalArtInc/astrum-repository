

import React from 'react';
import classNames from 'classnames';
import { NxList } from '@globalart/react-shared-components';
import DirectoryListItem from './DirectoryListItem';

import './DirectoryList.scss';

export function DirectoryList({ className, ...props }) {
  return <NxList className={classNames('as-directory-list', className)} {...props} />;
}

DirectoryList.DirectoryListItem = DirectoryListItem;
