

import { useEffect } from "react";

/*
 * This hook prevents the pushState method from being called with a hash (#) as the URL.
 * it's is to prevent a bug caused by zenscroll combined with the hash before the path in the URL.
 * 
 * Related spike https://globalart.atlassian.net/browse/ASTRUM-46710?focusedCommentId=762520
 */
export default function usePreventPushStateOnHash() {
  useEffect(() => {
    if (!window?.history) {
      return;
    }

    history.pushState = function () {
      console.debug("Intercepting pushState", arguments);
      if (arguments?.[2] !== "#") {
        History.prototype.pushState.apply(history, arguments);
      }
    };

    return () => {
      history.pushState = History.prototype.pushState;
    };
  }, []);
}
