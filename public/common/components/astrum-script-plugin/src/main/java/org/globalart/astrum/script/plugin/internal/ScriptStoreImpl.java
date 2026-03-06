
package org.globalart.astrum.script.plugin.internal;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.script.Script;
import org.globalart.astrum.transaction.Transactional;

import com.google.common.collect.ImmutableList;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * MyBatis {@link ScriptStore} implementation.
 *
 * @since 3.21
 */
@Component
@Qualifier("mybatis")
public class ScriptStoreImpl
    extends ConfigStoreSupport<ScriptDAO>
    implements ScriptStore
{
  @Autowired
  public ScriptStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Override
  public Script newScript() {
    return new ScriptData();
  }

  @Transactional
  @Override
  public List<Script> list() {
    return ImmutableList.copyOf(dao().browse());
  }

  @Transactional
  @Override
  public Script get(final String name) {
    return dao().read(name).orElse(null);
  }

  @Transactional
  @Override
  public void create(final Script script) {
    dao().create((ScriptData) script);
  }

  @Transactional
  @Override
  public void update(final Script script) {
    dao().update((ScriptData) script);
  }

  @Transactional
  @Override
  public void delete(final Script script) {
    dao().delete(script.getName());
  }
}
