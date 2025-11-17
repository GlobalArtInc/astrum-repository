
package org.globalart.astrum.internal.security.model;

import java.util.ArrayList;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataStoreManager;
import org.globalart.astrum.testdb.DataSessionRule;

import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableSet;
import com.google.common.collect.Iterables;
import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static java.util.Collections.emptyMap;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.nullValue;

public class CPrivilegeDAOTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(CPrivilegeDAO.class);

  private DataSession session;

  private CPrivilegeDAO dao;

  @Before
  public void setup() {
    session = sessionRule.openSession(DataStoreManager.DEFAULT_DATASTORE_NAME);
    dao = (CPrivilegeDAO) session.access(CPrivilegeDAO.class);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void testCreateReadUpdateDelete() {
    CPrivilegeData privilege = new CPrivilegeData();
    privilege.setId("notes-read");
    privilege.setName("app-notes-read");
    privilege.setDescription("Notes Read");
    privilege.setType("application");
    privilege.setProperties(ImmutableMap.of("domain", "notes", "actions", "view"));
    privilege.setReadOnly(true);
    dao.create(privilege);

    CPrivilegeData read = dao.read("notes-read").get();

    assertThat(read, not(nullValue()));
    assertThat(read.getId(), is("notes-read"));
    assertThat(read.getName(), is("app-notes-read"));
    assertThat(read.getDescription(), is("Notes Read"));
    assertThat(read.getType(), is("application"));
    assertThat(read.getProperties(), is(ImmutableMap.of("domain", "notes", "actions", "view")));
    assertThat(read.isReadOnly(), is(true));

    privilege.setName("app-notes-read2");
    privilege.setDescription("Notes Read2");
    privilege.setType("application2");
    privilege.setProperties(ImmutableMap.of("domain", "apps", "actions", "read,view"));
    privilege.setReadOnly(false);
    dao.update(privilege);

    read = dao.read("notes-read").get();

    assertThat(read, not(nullValue()));
    assertThat(read.getName(), is("app-notes-read2"));
    assertThat(read.getDescription(), is("Notes Read2"));
    assertThat(read.getType(), is("application2"));
    assertThat(read.getProperties(), is(ImmutableMap.of("domain", "apps", "actions", "read,view")));
    assertThat(read.isReadOnly(), is(false));

    dao.delete("notes-read");

    assertThat(dao.read("notes-read").isPresent(), is(false));
  }

  @Test
  public void testBrowse() {
    Set<String> ids = ImmutableSet.of("1", "2", "3");
    List<CPrivilegeData> privileges = generatePrivileges(ids);
    privileges.forEach(privilege -> dao.create(privilege));

    assertThat(Iterables.size(dao.browse()), is(ids.size()));
  }

  @Test
  public void testUpdate_returnsStatus() {
    CPrivilegeData privilege = new CPrivilegeData();
    privilege.setId("privilege1");
    privilege.setName("Privilege1");
    privilege.setDescription("Privilege 1");
    privilege.setType("Application");
    privilege.setProperties(emptyMap());
    privilege.setReadOnly(false);

    assertThat(dao.update(privilege), is(false));

    dao.create(privilege);
    privilege.setDescription("Privilege 2");

    assertThat(dao.update(privilege), is(true));
  }

  @Test
  public void testFindByIds() {
    Set<String> ids = ImmutableSet.of("1", "2", "3");
    List<CPrivilegeData> privileges = generatePrivileges(ids);
    privileges.forEach(privilege -> dao.create(privilege));

    assertThat(dao.findByIds(ids).size(), is(ids.size()));
  }

  private List<CPrivilegeData> generatePrivileges(final Set<String> ids) {
    List<CPrivilegeData> privileges = new ArrayList<>(ids.size());
    for (String id : ids) {
      CPrivilegeData privilege = new CPrivilegeData();
      privilege.setId(id);
      privilege.setName("Privilege" + id);
      privilege.setDescription("Privilege " + id);
      privilege.setType("Application");
      privilege.setProperties(emptyMap());
      privilege.setReadOnly(false);

      privileges.add(privilege);
    }

    return privileges;
  }
}
