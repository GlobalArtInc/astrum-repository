
package org.globalart.astrum.repository.apt.datastore.data;

import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.content.kv.FormatKeyValueConfigurationSupport;

import org.springframework.beans.factory.annotation.Qualifier;

@Qualifier(AptFormat.NAME)
public class AptKeyValueStoreProvider
    extends FormatKeyValueConfigurationSupport<AptKeyValueStore>
{
  // nothing to add
}
