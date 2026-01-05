
package org.globalart.astrum.security.user;

import org.globalart.astrum.security.role.RoleIdentifier;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@Component
@Qualifier("MockUserManagerA")
public class MockUserManagerA
    extends MockUserManagerSupport
{
  public MockUserManagerA() {
    super(new NoOpEventManager());
    User a = new User();
    a.setName("Joe Coder");
    a.setEmailAddress("jcoder@globalart.dev");
    a.setSource(this.getSource());
    a.setUserId("jcoder");
    a.addRole(new RoleIdentifier(this.getSource(), "RoleA"));
    a.addRole(new RoleIdentifier(this.getSource(), "RoleB"));
    a.addRole(new RoleIdentifier(this.getSource(), "RoleC"));

    User b = new User();
    b.setName("Christine H. Dugas");
    b.setEmailAddress("cdugas@globalart.dev");
    b.setSource(this.getSource());
    b.setUserId("cdugas");
    b.addRole(new RoleIdentifier(this.getSource(), "RoleA"));
    b.addRole(new RoleIdentifier(this.getSource(), "RoleB"));
    b.addRole(new RoleIdentifier(this.getSource(), "Role1"));

    User c = new User();
    c.setName("Patricia P. Peralez");
    c.setEmailAddress("pperalez@globalart.dev");
    c.setSource(this.getSource());
    c.setUserId("pperalez");

    User d = new User();
    d.setName("Danille S. Knudsen");
    d.setEmailAddress("dknudsen@globalart.dev");
    d.setSource(this.getSource());
    d.setUserId("dknudsen");

    User e = new User();
    e.setName("Anon e Mous");
    e.setEmailAddress("anonymous@globalart.dev");
    e.setSource(this.getSource());
    e.setUserId("anonymous-user");

    this.addUser(a, a.getUserId());
    this.addUser(b, b.getUserId());
    this.addUser(c, c.getUserId());
    this.addUser(d, d.getUserId());
    this.addUser(e, e.getUserId());
  }

  public String getSource() {
    return "MockUserManagerA";
  }

  public String getAuthenticationRealmName() {
    return "MockRealmA";
  }
}
