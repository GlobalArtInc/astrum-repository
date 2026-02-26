
package org.globalart.astrum.repository.security.internal;

import java.util.List;
import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.security.VariableResolverAdapterManager;

import com.google.common.annotations.VisibleForTesting;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Default implementation of {@link VariableResolverAdapterManager}.
 * 
 * @since 3.1
 */
@Component
public class VariableResolverAdapterManagerImpl
    implements VariableResolverAdapterManager
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @VisibleForTesting
  static final String DEFAULT_ADAPTER_NAME = "simple";

  private final VariableResolverAdapter defaultAdapter;

  private final Map<String, VariableResolverAdapter> adaptersByFormat;

  @Autowired
  public VariableResolverAdapterManagerImpl(final List<VariableResolverAdapter> adaptersByFormatList) {
    this.adaptersByFormat = QualifierUtil.buildQualifierBeanMap(checkNotNull(adaptersByFormatList));
    this.defaultAdapter = checkNotNull(this.adaptersByFormat.get(DEFAULT_ADAPTER_NAME));
  }

  @Override
  public VariableResolverAdapter get(String format) {
    return adaptersByFormat.getOrDefault(format, defaultAdapter);
  }
}
