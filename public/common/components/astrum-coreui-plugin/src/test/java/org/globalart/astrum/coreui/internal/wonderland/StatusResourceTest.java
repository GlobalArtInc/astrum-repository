
package org.globalart.astrum.coreui.internal.wonderland;

import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEdition;
import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionSelector;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class StatusResourceTest
{
  @Mock
  private ApplicationVersion applicationVersion;

  @Mock
  private AstrumEditionSelector astrumEditionSelector;

  @Mock
  private AstrumEdition astrumEdition;

  private StatusResource underTest;

  @Test
  void getReturnsVersionAndEdition() {
    when(applicationVersion.getVersion()).thenReturn("3.70.0-01");
    when(applicationVersion.getEdition()).thenReturn("PRO");
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("astrum-professional-edition");

    underTest = new StatusResource(applicationVersion, astrumEditionSelector);
    StatusXO result = underTest.get();

    assertThat(result, is(notNullValue()));
    assertThat(result.getVersion(), is("3.70.0-01"));
    assertThat(result.getEdition(), is("PRO"));
    assertThat(result.getIsCloud(), is(false));
    verify(applicationVersion).getVersion();
    verify(applicationVersion).getEdition();
  }

  @Test
  void getReturnsOssEdition() {
    when(applicationVersion.getVersion()).thenReturn("3.70.0-01");
    when(applicationVersion.getEdition()).thenReturn("OSS");
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("astrum-core-edition");

    underTest = new StatusResource(applicationVersion, astrumEditionSelector);
    StatusXO result = underTest.get();

    assertThat(result, is(notNullValue()));
    assertThat(result.getVersion(), is("3.70.0-01"));
    assertThat(result.getEdition(), is("OSS"));
    assertThat(result.getIsCloud(), is(false));
  }

  @Test
  void getReturnsIsCloudTrueForCloudEdition() {
    when(applicationVersion.getVersion()).thenReturn("3.70.0-01");
    when(applicationVersion.getEdition()).thenReturn("PRO");
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("astrum-cloud-edition");

    underTest = new StatusResource(applicationVersion, astrumEditionSelector);
    StatusXO result = underTest.get();

    assertThat(result, is(notNullValue()));
    assertThat(result.getVersion(), is("3.70.0-01"));
    assertThat(result.getEdition(), is("PRO"));
    assertThat(result.getIsCloud(), is(true));
  }

  @Test
  void getHandlesNullValues() {
    when(applicationVersion.getVersion()).thenReturn(null);
    when(applicationVersion.getEdition()).thenReturn(null);
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("astrum-professional-edition");

    underTest = new StatusResource(applicationVersion, astrumEditionSelector);
    StatusXO result = underTest.get();

    assertThat(result, is(notNullValue()));
    assertThat(result.getVersion(), is((String) null));
    assertThat(result.getEdition(), is((String) null));
    assertThat(result.getIsCloud(), is(false));
  }
}
