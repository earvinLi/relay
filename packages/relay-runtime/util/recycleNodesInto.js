/**
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 * @flow strict
 * @format
 */

// flowlint ambiguous-object-type:error

'use strict';

/**
 * Recycles subtrees from `prevData` by replacing equal subtrees in `nextData`.
 */
function recycleNodesInto<T>(prevData: T, nextData: T): T {
  if (
    prevData === nextData ||
    typeof prevData !== 'object' ||
    prevData instanceof Set ||
    prevData instanceof WeakSet ||
    prevData instanceof Map ||
    prevData instanceof WeakMap ||
    !prevData ||
    typeof nextData !== 'object' ||
    nextData instanceof WeakSet ||
    nextData instanceof Set ||
    nextData instanceof WeakMap ||
    nextData instanceof Map ||
    !nextData
  ) {
    return nextData;
  }
  let canRecycle = false;

  // Assign local variables to preserve Flow type refinement.
  const prevArray = Array.isArray(prevData) ? prevData : null;
  const nextArray = Array.isArray(nextData) ? nextData : null;
  if (prevArray && nextArray) {
    canRecycle =
      nextArray.reduce((wasEqual, nextItem, ii) => {
        const prevValue = prevArray[ii];
        const nextValue = recycleNodesInto(prevValue, nextItem);
        if (nextValue !== nextArray[ii]) {
          if (__DEV__) {
            if (!Object.isFrozen(nextArray)) {
              nextArray[ii] = nextValue;
            }
          } else {
            nextArray[ii] = nextValue;
          }
        }
        return wasEqual && nextValue === prevArray[ii];
      }, true) && prevArray.length === nextArray.length;
  } else if (!prevArray && !nextArray) {
    // Assign local variables to preserve Flow type refinement.
    const prevObject = prevData;
    const nextObject = nextData;
    const prevKeys = Object.keys(prevObject);
    const nextKeys = Object.keys(nextObject);
    canRecycle =
      nextKeys.reduce((wasEqual, key) => {
        const prevValue = prevObject[key];
        const nextValue = recycleNodesInto(prevValue, nextObject[key]);
        if (nextValue !== nextObject[key]) {
          if (__DEV__) {
            if (!Object.isFrozen(nextObject)) {
              // $FlowFixMe[cannot-write]
              nextObject[key] = nextValue;
            }
          } else {
            // $FlowFixMe[cannot-write]
            nextObject[key] = nextValue;
          }
        }
        return wasEqual && nextValue === prevObject[key];
      }, true) && prevKeys.length === nextKeys.length;
  }
  return canRecycle ? prevData : nextData;
}

module.exports = recycleNodesInto;
