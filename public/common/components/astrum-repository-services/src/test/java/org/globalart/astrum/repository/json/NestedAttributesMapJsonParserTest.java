
package org.globalart.astrum.repository.json;

import java.io.IOException;

import org.globalart.astrum.common.collect.NestedAttributesMap;

import com.fasterxml.jackson.core.JsonFactory;
import com.fasterxml.jackson.core.JsonParser;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static com.google.common.collect.Maps.newHashMap;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.Assert.assertTrue;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class NestedAttributesMapJsonParserTest
{
  @Mock
  private JsonParser jsonParser;

  private NestedAttributesMap nestedAttributesMap = new NestedAttributesMap("testMap", newHashMap());

  private NestedAttributesMapJsonParser underTest;

  @Before
  public void setUp() {
    underTest = new NestedAttributesMapJsonParser(jsonParser, nestedAttributesMap);
  }

  @Test
  public void default_Mapping_Disabled_By_Default() {
    assertFalse(underTest.isDefaultMapping());
  }

  @Test
  public void change_defaultMapping() {
    underTest.enableDefaultMapping();
    assertTrue(underTest.isDefaultMapping());

    underTest.disableDefaultMapping();
    assertFalse(underTest.isDefaultMapping());
  }

  @Test
  public void mapping_Inside_Array_Disabled_By_Default() {
    assertFalse(underTest.isMappingInsideArray());
  }

  @Test
  public void mark_MappingInsideArray() {
    underTest.markMappingInsideArray();
    assertTrue(underTest.isMappingInsideArray());

    underTest.unMarkMappingInsideArray();
    assertFalse(underTest.isMappingInsideArray());
  }

  @Test
  public void assure_Same_RootMap() {
    assertEquals(underTest.getRoot(), nestedAttributesMap);
  }

  @Test
  public void no_ChildMap_From_RootMap_On_MappingInsideArray() {
    underTest.markMappingInsideArray();
    assertThat(underTest.getChildFromRoot(), nullValue());
  }

  @Test
  public void no_ChildMap_From_RootMap_On_NoChildFound() {
    assertThat(underTest.getChildFromRoot(), nullValue());
  }

  @Test
  public void retrieve_ChildMap_From_RootMap() throws IOException {
    String simpleJson = "{\"user\":{\"description\":\"simplestuff\"}}";
    underTest = new NestedAttributesMapJsonParser(new JsonFactory().createParser(simpleJson), nestedAttributesMap);

    // for the test fast forward to first chid
    underTest.nextValue();
    underTest.nextValue();
    NestedAttributesMap child = underTest.getChildFromRoot();
    assertThat(child.getKey(), equalTo("user"));

    underTest.nextValue();
    child = underTest.getChildFromRoot();
    assertThat(child.getKey(), equalTo("description"));
  }
}
