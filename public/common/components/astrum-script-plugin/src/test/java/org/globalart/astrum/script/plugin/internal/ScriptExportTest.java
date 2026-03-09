
package org.globalart.astrum.script.plugin.internal;

import java.io.File;
import java.io.IOException;
import java.util.Arrays;
import java.util.List;

import org.globalart.astrum.script.Script;
import org.globalart.astrum.supportzip.datastore.JsonExporter;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.Matchers.anyOf;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Tests validity of Serialization/Deserialization {@link Script} by {@link ScriptExport}
 */
public class ScriptExportTest
{
  private final JsonExporter jsonExporter = new JsonExporter();

  private File jsonFile;

  @Before
  public void setup() throws IOException {
    jsonFile = File.createTempFile("SamlUser", ".json");
  }

  @After
  public void tearDown() {
    jsonFile.delete();
  }

  @Test
  public void testExportImportToJson() throws Exception {
    List<Script> scripts = Arrays.asList(
        createScript("script_1"),
        createScript("script_2"));

    ScriptStore store = mock(ScriptStore.class);
    when(store.list()).thenReturn(scripts);

    ScriptExport exporter = new ScriptExport(store);
    exporter.export(jsonFile);
    List<ScriptData> importedData = jsonExporter.importFromJson(jsonFile, ScriptData.class);

    assertThat(importedData.size(), is(2));
    importedData.forEach(data -> assertThat(data.getName(), anyOf(
        is(scripts.get(0).getName()),
        is(scripts.get(1).getName()))));
    importedData.forEach(data -> assertThat(data.getType(), is("script")));
    importedData.forEach(data -> assertThat(data.getContent(), is("log.info('world')")));
  }

  private Script createScript(final String name) {
    ScriptData script = new ScriptData();
    script.setName(name);
    script.setType("script");
    script.setContent("log.info('world')");

    return script;
  }
}
