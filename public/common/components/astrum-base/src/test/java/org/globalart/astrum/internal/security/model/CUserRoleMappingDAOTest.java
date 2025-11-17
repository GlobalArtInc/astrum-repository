
package org.globalart.astrum.internal.security.model;

import java.util.stream.Stream;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataStoreManager;
import org.globalart.astrum.testdb.DataSessionRule;

import com.google.common.collect.Iterables;
import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static java.util.Collections.emptySet;
import static java.util.Collections.singleton;
import static java.util.stream.Collectors.toSet;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.nullValue;

public class CUserRoleMappingDAOTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(CUserRoleMappingDAO.class);

  private DataSession session;

  private CUserRoleMappingDAO dao;

  @Before
  public void setup() {
    session = sessionRule.openSession(DataStoreManager.DEFAULT_DATASTORE_NAME);
    dao = (CUserRoleMappingDAO) session.access(CUserRoleMappingDAO.class);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void testCreateReadUpdateDelete_caseSensitive() {
    CUserRoleMappingData roleMapping = new CUserRoleMappingData();
    roleMapping.setUserId("admin");
    roleMapping.setSource("other");
    roleMapping.setRoles(Stream.of("role1", "role2").collect(toSet()));
    dao.create(roleMapping);

    assertThat(dao.read("ADMIN", "other").isPresent(), is(false));

    CUserRoleMappingData read = dao.read("admin", "other").get();

    assertThat(read, not(nullValue()));
    assertThat(read.getUserId(), is("admin"));
    assertThat(read.getSource(), is("other"));
    assertThat(read.getRoles(), is(Stream.of("role1", "role2").collect(toSet())));

    roleMapping.setUserId("ADMIN");
    roleMapping.setRoles(singleton("role3"));
    dao.update(roleMapping);

    read = dao.read("admin", "other").get();

    assertThat(read, not(nullValue()));
    assertThat(read.getUserId(), is("admin"));
    assertThat(read.getSource(), is("other"));
    assertThat(read.getRoles(), is(Stream.of("role1", "role2").collect(toSet())));

    roleMapping.setUserId("admin");
    dao.update(roleMapping);

    read = dao.read("admin", "other").get();
    assertThat(read.getRoles(), is(singleton("role3")));

    assertThat(dao.delete("Admin", "other"), is(false));
    assertThat(dao.read("admin", "other").isPresent(), is(true));

    assertThat(dao.delete("admin", "other"), is(true));
    assertThat(dao.read("admin", "other").isPresent(), is(false));
  }

  @Test
  public void testBrowse() {
    CUserRoleMappingData roleMapping1 = new CUserRoleMappingData();
    roleMapping1.setUserId("user1");
    roleMapping1.setSource("default");
    roleMapping1.setRoles(emptySet());
    CUserRoleMappingData roleMapping2 = new CUserRoleMappingData();
    roleMapping2.setUserId("user2");
    roleMapping2.setSource("default");
    roleMapping2.setRoles(emptySet());
    CUserRoleMappingData roleMapping3 = new CUserRoleMappingData();
    roleMapping3.setUserId("user3");
    roleMapping3.setSource("default");
    roleMapping3.setRoles(emptySet());

    dao.create(roleMapping1);
    dao.create(roleMapping2);
    dao.create(roleMapping3);

    assertThat(Iterables.size(dao.browse()), is(3));
  }

  @Test
  public void testUpdate_returnsStatus() {
    CUserRoleMappingData roleMapping = new CUserRoleMappingData();
    roleMapping.setUserId("user1");
    roleMapping.setSource("default");
    roleMapping.setRoles(emptySet());

    assertThat(dao.update(roleMapping), is(false));

    dao.create(roleMapping);
    roleMapping.setRoles(singleton("role1"));

    assertThat(dao.update(roleMapping), is(true));
  }
}
