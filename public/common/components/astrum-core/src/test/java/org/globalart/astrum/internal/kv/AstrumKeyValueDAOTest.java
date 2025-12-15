
package org.globalart.astrum.internal.kv;

import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Supplier;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.kv.AstrumKeyValue;
import org.globalart.astrum.kv.ValueType;
import org.globalart.astrum.testdb.DataSessionRule;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.junit.Assert.assertTrue;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class AstrumKeyValueDAOTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule()
      .access(AstrumKeyValueDAO.class);

  @Test
  public void testStringValue() {
    AstrumKeyValue kv = new AstrumKeyValue();
    kv.setKey("boolean-test");
    kv.setType(ValueType.CHARACTER);
    kv.setValue("test-value");

    assertKeyValueOperations(() -> kv, (existing) -> assertThat(existing.getAsString(), is("test-value")));
  }

  @Test
  public void testIntValue() {
    AstrumKeyValue kv = new AstrumKeyValue();
    kv.setKey("int-test");
    kv.setType(ValueType.NUMBER);
    kv.setValue(10);

    assertKeyValueOperations(() -> kv, (existing) -> assertThat(existing.getAsInt(), is(10)));
  }

  @Test
  public void testBooleanValue() {
    AstrumKeyValue kv = new AstrumKeyValue();
    kv.setKey("boolean-test");
    kv.setType(ValueType.BOOLEAN);
    kv.setValue(false);

    assertKeyValueOperations(() -> kv, (existing) -> assertThat(existing.getAsBoolean(), is(false)));
  }

  @Test
  public void testObjectValue() {
    TestObject objectValue = new TestObject("test", 23, true);

    AstrumKeyValue kv = new AstrumKeyValue();
    kv.setKey("object-test");
    kv.setType(ValueType.OBJECT);
    kv.setValue(objectValue);

    assertKeyValueOperations(() -> kv, (existing) -> {
      ObjectMapper mapper = new ObjectMapper();
      assertThat(existing.getAsObject(mapper, TestObject.class), equalTo(objectValue));
    });
  }

  @Test
  public void testObjectList() {
    TestObject object1 = new TestObject("object-1", 32, false);
    TestObject object2 = new TestObject("object-2", 65, true);

    AstrumKeyValue kv = new AstrumKeyValue();
    kv.setKey("object-list-test");
    kv.setType(ValueType.OBJECT);
    kv.setValue(Arrays.asList(object1, object2));

    assertKeyValueOperations(() -> kv, (existing) -> {
      ObjectMapper mapper = new ObjectMapper();
      List<TestObject> objectList = existing.getAsObjectList(mapper, TestObject.class);
      assertThat(objectList, hasSize(2));
      assertThat(objectList.get(0), equalTo(object1));
      assertThat(objectList.get(1), equalTo(object2));
    });
  }

  private void assertKeyValueOperations(
      final Supplier<AstrumKeyValue> kvSupplier,
      final Consumer<AstrumKeyValue> assertionConsumer)
  {
    AstrumKeyValue initial = kvSupplier.get();
    write((dao) -> dao.set(initial));

    Optional<AstrumKeyValue> existing = read((dao) -> dao.get(initial.key()));

    // verify kv can be read correctly
    assertTrue(existing.isPresent());
    assertKeyValue(initial, existing.get());
    // pass to consumer with specific kv assertions
    assertionConsumer.accept(existing.get());

    // verify kv can be deleted correctly
    boolean deleted = read((dao) -> dao.remove(initial.key()));
    assertTrue(deleted);
  }

  private void assertKeyValue(final AstrumKeyValue a, final AstrumKeyValue b) {
    assertThat(a.key(), is(b.key()));
    assertThat(a.type(), is(b.type()));
  }

  private <R> R read(final Function<AstrumKeyValueDAO, R> function) {
    try (DataSession<?> dataSession = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      AstrumKeyValueDAO dao = dataSession.access(AstrumKeyValueDAO.class);
      R result = function.apply(dao);

      dataSession.getTransaction().commit();
      return result;
    }
  }

  private void write(final Consumer<AstrumKeyValueDAO> consumer) {
    try (DataSession<?> dataSession = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      AstrumKeyValueDAO dao = dataSession.access(AstrumKeyValueDAO.class);
      consumer.accept(dao);
      dataSession.getTransaction().commit();
    }
  }

  static class TestObject
  {
    private String name;

    private int age;

    private boolean exclusive;

    public TestObject(final String name, final int age, final boolean exclusive) {
      this.name = name;
      this.age = age;
      this.exclusive = exclusive;
    }

    public TestObject() {

    }

    public String getName() {
      return name;
    }

    public void setName(final String name) {
      this.name = name;
    }

    public int getAge() {
      return age;
    }

    public void setAge(final int age) {
      this.age = age;
    }

    public boolean isExclusive() {
      return exclusive;
    }

    public void setExclusive(final boolean exclusive) {
      this.exclusive = exclusive;
    }

    @Override
    public boolean equals(final Object o) {
      if (this == o) {
        return true;
      }
      if (o == null || getClass() != o.getClass()) {
        return false;
      }
      TestObject that = (TestObject) o;
      return age == that.age && exclusive == that.exclusive && Objects.equals(name, that.name);
    }

    @Override
    public int hashCode() {
      return Objects.hash(name, age, exclusive);
    }
  }
}
