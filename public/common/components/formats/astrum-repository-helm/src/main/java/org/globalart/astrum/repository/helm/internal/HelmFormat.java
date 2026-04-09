
package org.globalart.astrum.repository.helm.internal;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import java.util.List;


import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.Format;

import com.google.common.collect.ImmutableList;

import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;

/**
 * Helm repository format.
 */
@Component
@Qualifier(HelmFormat.NAME)
public class HelmFormat
    extends Format
{
  public static final String NAME = "helm";

  public static final List<HashAlgorithm> HASH_ALGORITHMS = ImmutableList.of(SHA1, SHA256);

  public HelmFormat() {
    super(NAME);
  }
}
