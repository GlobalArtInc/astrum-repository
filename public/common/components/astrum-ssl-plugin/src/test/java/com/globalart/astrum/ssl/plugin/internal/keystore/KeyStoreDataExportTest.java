
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.Arrays;
import java.util.List;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.allOf;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link KeyStoreDataExport}
 */
@ExtendWith(MockitoExtension.class)
class KeyStoreDataExportTest
{
  @Mock
  private PersistentKeyStoreStorageManager keyStoreStorageManager;

  @Captor
  private ArgumentCaptor<KeyStoreData> keyStoreDataCaptor;

  @InjectMocks
  private KeyStoreDataExport underTest;

  private File jsonFile;

  @TempDir
  File tempDir;

  @BeforeEach
  void setup() {
    jsonFile = new File(tempDir, "key_store_data.json");
  }

  @Test
  void testExport() throws Exception {
    List<KeyStoreData> keyStoreDataList = Arrays.asList(
        createKeyStoreData("trusted.ks"),
        createKeyStoreData("private.ks"));

    when(keyStoreStorageManager.browse()).thenReturn(keyStoreDataList);

    underTest.export(jsonFile);

    assertThat(jsonFile.exists(), is(true));

    String jsonContent = Files.readString(jsonFile.toPath());
    assertThat(jsonContent, allOf(
        containsString("trusted.ks"),
        containsString("private.ks")));
  }

  @Test
  void testRestore() throws Exception {
    KeyStoreData trustedKs = createKeyStoreData("trusted.ks");
    KeyStoreData privateKs = createKeyStoreData("private.ks");

    when(keyStoreStorageManager.browse()).thenReturn(List.of(trustedKs, privateKs));

    underTest.export(jsonFile);

    underTest.restore(jsonFile);

    verify(keyStoreStorageManager, times(2)).save(keyStoreDataCaptor.capture());
    List<KeyStoreData> capturedData = keyStoreDataCaptor.getAllValues();

    KeyStoreData capturedTrustedKs = capturedData.get(0);
    assertThat(capturedTrustedKs.getName(), is(trustedKs.getName()));
    assertThat(Arrays.equals(capturedTrustedKs.getBytes(), trustedKs.getBytes()), is(true));

    KeyStoreData capturedPrivateKs = capturedData.get(1);
    assertThat(capturedPrivateKs.getName(), is(privateKs.getName()));
    assertThat(Arrays.equals(capturedPrivateKs.getBytes(), privateKs.getBytes()), is(true));
  }

  private KeyStoreData createKeyStoreData(final String name) {
    KeyStoreData data = new KeyStoreData();
    data.setName(name);
    data.setBytes(("test-bytes-for-" + name).getBytes(StandardCharsets.UTF_8));
    return data;
  }
}
