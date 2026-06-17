
package org.globalart.astrum.coreui.internal.wonderland;

import java.util.List;

import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class SettingsResourceTest
{
  @Test
  void getReturnsKeepAliveProperty() {
    SettingsResource underTest = new SettingsResource();

    List<PropertyXO> properties = underTest.get();

    assertThat(properties, is(notNullValue()));
    assertThat(properties, hasSize(1));
    assertThat(properties.get(0).getKey(), is("keepAlive"));
    // Default value for astrum.ui.keepAlive is true
    assertThat(properties.get(0).getValue(), is("true"));
  }

  @Test
  void getReturnsNonEmptyList() {
    SettingsResource underTest = new SettingsResource();

    List<PropertyXO> properties = underTest.get();

    assertThat(properties.isEmpty(), is(false));
  }
}
