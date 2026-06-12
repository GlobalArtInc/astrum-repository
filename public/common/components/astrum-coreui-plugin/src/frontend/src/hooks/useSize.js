
import {useLayoutEffect, useState} from 'react';
import useResizeObserver from '@react-hook/resize-observer';

/**
 * Track size changes to a React RefObject
 * @param target RefObject
 * @returns {{width: number, height: number}}
 */
export function useSize(target) {
  const [size, setSize] = useState({width: 0, height: 0});

  useLayoutEffect(() => {
    if (target.current) {
      setSize(target.current.getBoundingClientRect());
    }
  }, [target]);

  useResizeObserver(target, (entry) => {
    setSize(entry.target.getBoundingClientRect())
  });

  return size;
}
