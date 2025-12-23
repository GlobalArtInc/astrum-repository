
package org.globalart.astrum.security.config;

import org.globalart.astrum.security.privilege.WildcardPrivilegeDescriptor;

public class MutableTestSecurityContributor
    extends MutableSecurityContributor
{
  private boolean configRequested = false;

  private static int INSTANCE_COUNT = 1;

  private String privId = "priv-" + INSTANCE_COUNT++;

  @Override
  protected void initial(final SecurityConfiguration model) {
    model.addPrivilege(WildcardPrivilegeDescriptor.privilege("foo:bar:" + privId + ":read"));
  }

  public String getId() {
    return privId;
  }

  public void setConfigRequested(boolean configRequested) {
    this.configRequested = configRequested;
  }

  public boolean wasConfigRequested() {
    return configRequested;
  }

  @Override
  public SecurityConfiguration getContribution() {
    setConfigRequested(true);
    return super.getContribution();
  }

  public void setDirty(boolean dirty) {
    if (dirty) {
      setConfigRequested(false);
      apply((model, configurationManager) -> {
        // marks model as dirty
      });
    }
  }
}
