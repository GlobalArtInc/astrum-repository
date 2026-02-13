
package org.globalart.astrum.repository.content.tasks;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

public class PrivilegeNxRM2UpgradeTaskDescriptorTest
{
  @Test
  public void shouldCreateInstanceWithDefaultValues() {
    PrivilegeNxRM2UpgradeTaskDescriptor underTest = new PrivilegeNxRM2UpgradeTaskDescriptor();

    assertThat(underTest.isVisible()).isFalse();
    assertThat(underTest.isExposed()).isFalse();
    assertThat(underTest.getId()).isEqualTo("nxrm2.privilege.upgrade");
    assertThat(underTest.getName()).isEqualTo("Upgrade NxRM2 privileges Task");
  }
}
