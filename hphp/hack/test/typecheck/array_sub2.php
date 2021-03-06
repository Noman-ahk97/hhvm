<?hh // partial
/**
 * Copyright (c) 2014, Facebook, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the "hack" directory of this source tree.
 *
 *
 */

function get_keys<T2>(array<bool, T2> $x): array<bool> {
  $result = array();
  foreach ($x as $k => $v) {
    $result[] = $k;
  }
  return $result;
}

function test(array<int> $a): void {
  get_keys($a);
}
