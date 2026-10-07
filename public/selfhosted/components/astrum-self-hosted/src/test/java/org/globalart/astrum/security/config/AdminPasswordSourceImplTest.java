
package org.globalart.astrum.security.config;

import java.io.IOException;

import org.globalart.astrum.security.internal.AdminPasswordFileManagerImpl;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AdminPasswordSourceImplTest
{
  @Mock
  private AdminPasswordFileManagerImpl adminPasswordFileManager;

  private AdminPasswordSourceImpl underTest;

  @Before
  public void setup() {
    underTest = new AdminPasswordSourceImpl(adminPasswordFileManager);
  }

  @Test
  public void shouldBePasswordFromFileWhenFileExists() throws Exception {
    when(adminPasswordFileManager.readFile()).thenReturn("password");

    assertThat(underTest.getPassword(false), is("password"));
    verify(adminPasswordFileManager, never()).writeFile(any());
  }

  @Test
  public void shouldBeDefaultPasswordWhenRandomIsFalse() throws Exception {
    assertThat(underTest.getPassword(false), is("admin123"));

    verify(adminPasswordFileManager, never()).writeFile(any());
  }

  @Test
  public void shouldBeRandomPasswordWhenRandomIsTruePasswordFileNotExistsAndPasswordFileWritten() throws Exception {
    when(adminPasswordFileManager.writeFile(any())).thenReturn(true);

    assertThat(underTest.getPassword(true), is(not("admin123")));

    verify(adminPasswordFileManager).writeFile(any());
  }

  @Test
  public void shouldBeDefaultPasswordWhenRandomIsTruePasswordFileNotExistsAndPasswordFileWriteFails() throws Exception {
    assertThat(underTest.getPassword(true), is("admin123"));

    verify(adminPasswordFileManager).writeFile(any());
  }

  /*
   * this test ensures that we don't inadvertently overwrite the serialized password from the first run
   */
  @Test
  public void testAdminPasswordIsWrittenOnlyOnce() throws IOException {
    when(adminPasswordFileManager.writeFile(any())).thenReturn(true);

    String password = underTest.getPassword(true);
    assertThat(password, is(notNullValue()));

    when(adminPasswordFileManager.readFile()).thenReturn(password);
    assertThat(underTest.getPassword(true), is(password));

    // should only write once
    verify(adminPasswordFileManager).writeFile(any());
  }
}
