
import React from 'react';

/**
 * @since 3.24
 */
export default function Detail({children, itemId, ...attrs}) {
  return <>{React.Children.map(children, (child) => React.cloneElement(child, {...attrs, itemId}))}</>;
}
