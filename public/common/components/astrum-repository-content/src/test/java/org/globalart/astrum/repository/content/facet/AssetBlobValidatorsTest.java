
package org.globalart.astrum.repository.content.facet;

import java.io.FileNotFoundException;
import java.util.Collections;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.mime.DefaultContentValidator;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.sameInstance;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyBoolean;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AssetBlobValidatorsTest
{
  private static final String FORMAT_VALUE = "raw";

  @Mock
  private Repository repository;

  @Mock
  private Format format;

  @Mock
  private DefaultContentValidator defaultContentValidator;

  @Mock
  private InputStreamSupplier contentSupplier;

  private AssetBlobValidators underTest;

  @Before
  public void setUp() {
    when(format.getValue()).thenReturn(FORMAT_VALUE);
    when(repository.getFormat()).thenReturn(format);

    // Construct with empty lists so the internal maps will be empty,
    // exercising the default fallback paths
    underTest = new AssetBlobValidators(
        Collections.emptyList(),
        Collections.emptyList(),
        defaultContentValidator);
  }

  @Test
  public void testSelectValidatorFallsBackToDefault() {
    AssetBlobValidator validator = underTest.selectValidator(repository);
    assertThat(validator, is(notNullValue()));
  }

  @Test
  public void testSelectedValidatorDeterminesContentType() throws Exception {
    String expectedContentType = "application/octet-stream";
    when(defaultContentValidator.determineContentType(
        anyBoolean(), any(InputStreamSupplier.class), any(MimeRulesSource.class), anyString(), anyString()))
            .thenReturn(expectedContentType);

    AssetBlobValidator validator = underTest.selectValidator(repository);

    String result = validator.determineContentType(true, contentSupplier, "/path/to/file.bin", "application/zip");

    assertThat(result, is(equalTo(expectedContentType)));
    verify(defaultContentValidator).determineContentType(
        eq(true), eq(contentSupplier), eq(MimeRulesSource.NOOP), eq("/path/to/file.bin"), eq("application/zip"));
  }

  @Test
  public void testSelectValidatorUsesFormatSpecificValidator() {
    // With empty lists, there are no format-specific validators,
    // so the default content validator is used as the fallback.
    // This test verifies the fallback behavior when no format-specific validators exist.
    AssetBlobValidator validator = underTest.selectValidator(repository);
    assertThat(validator, is(notNullValue()));
  }

  @Test
  public void testSelectedValidatorWrapsIOExceptionWithSafeMessage() throws Exception {
    // The cause's message may contain a filesystem path (e.g. FileNotFoundException); the wrapping
    // InvalidContentException must not expose that path to callers that surface getMessage() to clients.
    FileNotFoundException cause = new FileNotFoundException("/opt/globalart-work/astrum3/blobs/default/content/secret");
    when(defaultContentValidator.determineContentType(
        anyBoolean(), any(InputStreamSupplier.class), any(MimeRulesSource.class), anyString(), anyString()))
            .thenThrow(cause);

    AssetBlobValidator validator = underTest.selectValidator(repository);

    InvalidContentException thrown = assertThrows(InvalidContentException.class,
        () -> validator.determineContentType(true, contentSupplier, "/path/to/file.bin", "application/zip"));

    assertThat(thrown.getMessage(), is(equalTo("Content type could not be determined")));
    assertThat(thrown.getMessage(), is(not(equalTo(cause.toString()))));
    assertThat(thrown.getCause(), is(sameInstance(cause)));
  }
}
