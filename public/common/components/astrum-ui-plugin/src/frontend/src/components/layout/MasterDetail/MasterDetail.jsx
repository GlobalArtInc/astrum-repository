
import classNames from 'classnames';
import React, {Children, cloneElement, useState} from 'react';
import { useRouter } from '@uirouter/react';
import PropTypes from 'prop-types';

import ExtJS from '../../../interface/ExtJS';

import Master from './Master';
import Detail from './Detail';
import { AsPageMain } from '@globalart/react-shared-components';

/**
 * @since 3.24
 */
export default function MasterDetail({className, children, path, ...attrs}) {
  const {location: {pathname}} = ExtJS.useHistory({basePath: path});
  const currentPath = useRouter().urlService.path();
  const [isCreate, setCreate] = useState(false);
  const [response, setResponse] = useState(null);

  const classes = classNames('as-master-detail', className);
  const isMasterRoute = pathname === '';
  // Re-encode itemId after browser decoding to handle special characters (/, ?, &, #, +, spaces)
  // Pattern: encodeURIComponent(decodeURIComponent(x)) ensures consistent encoding regardless
  // of whether the input is already encoded or decoded
  const itemId = isMasterRoute ? '' : encodeURIComponent(decodeURIComponent(pathname.substring(1)));

  if (!currentPath?.includes(path)) {
    return null;
  }

  function onCreate() {
    setCreate(true);
  }

  function onEdit(itemId) {
    // Ensure itemId is properly encoded to handle special characters like slashes
    const encodedItemId = encodeURIComponent(decodeURIComponent(itemId));
    window.location.hash = `${path}:${encodedItemId}`;
  }

  function onDone(doneResponse) {
    setCreate(false);
    const warning = doneResponse?.evt?.data?.data;
    if (warning && typeof warning == 'string') {
      setResponse(warning);
    } else {
      setResponse(null);
    }
    window.location.hash = path;
  }

  const childrenArray = Children.toArray(children);
  const master = cloneElement(childrenArray.filter(child => child.type === Master)[0], {
    onCreate,
    onEdit,
    response,
    setResponse
  });
  const detail = cloneElement(childrenArray.filter(child => child.type === Detail)[0], {
    itemId,
    onDone
  });

  return (
    <AsPageMain className={classes} {...attrs}>
      {isMasterRoute && !isCreate ? master : null}
      {!isMasterRoute || isCreate ? detail : null}
    </AsPageMain>
  );
}

MasterDetail.propTypes = {
  path: PropTypes.string.isRequired
};
