
package org.globalart.astrum.security;

import java.util.ArrayList;
import java.util.Arrays;

import org.globalart.astrum.audit.AuditRecorder;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.bootstrap.entrypoint.jetty.JettyServer;
import org.globalart.astrum.bootstrap.jetty.ManagedJetty;
import org.globalart.astrum.bootstrap.security.WebSecurityConfiguration;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.crypto.secrets.SecretsStore;
import org.globalart.astrum.security.AbstractSecurityTest.TestSecurityConfigurationMocks;
import org.globalart.astrum.security.anonymous.AnonymousManager;
import org.globalart.astrum.security.config.PreconfiguredSecurityConfigurationSource;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfigurationSource;
import org.globalart.astrum.security.internal.AuthorizingRealmImpl;
import org.globalart.astrum.security.realm.MemoryRealmConfigurationStore;
import org.globalart.astrum.security.realm.RealmConfiguration;
import org.globalart.astrum.security.realm.RealmConfigurationStore;
import org.globalart.astrum.security.realm.TestRealmConfiguration;
import org.globalart.astrum.security.user.UserManager;
import org.globalart.astrum.testcommon.event.SimpleEventManager;

import org.apache.shiro.authc.credential.PasswordService;
import org.apache.shiro.util.ThreadContext;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.BeanFactoryAnnotationUtils;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.context.ApplicationContext;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.ComponentScan.Filter;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.FilterType;
import org.springframework.context.annotation.Import;
import org.springframework.context.annotation.Primary;
import org.springframework.test.annotation.DirtiesContext;
import org.springframework.test.context.ContextConfiguration;
import org.springframework.test.context.bean.override.mockito.MockitoBean;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.when;
import static org.springframework.test.annotation.DirtiesContext.ClassMode.AFTER_CLASS;

@DirtiesContext(classMode = AFTER_CLASS)
@SpringBootTest
@ContextConfiguration(classes = TestSecurityConfigurationMocks.class)
@ComponentScan(basePackages = "org.globalart.astrum",
    excludeFilters = {@Filter(type = FilterType.ASSIGNABLE_TYPE, classes = {ManagedJetty.class, JettyServer.class})})
@Import({org.globalart.astrum.bootstrap.security.SecurityConfiguration.class, WebSecurityConfiguration.class})
public abstract class AbstractSecurityTest
{
  private static final Logger log = LoggerFactory.getLogger(AbstractSecurityTest.class);

  @Autowired
  protected ApplicationContext applicationContext;

  @MockitoBean
  protected PasswordService passwordService;

  @BeforeEach
  protected void setUp() throws Exception {
    getSecuritySystem().start();
  }

  @AfterEach
  protected void tearDown() throws Exception {
    try {
      getSecuritySystem().stop();
    }
    catch (Exception e) {
      log.warn("Failed to stop security-system", e);
    }

    ThreadContext.remove();
  }

  protected <T> T lookup(final Class<T> role) {
    return applicationContext.getBean(role);
  }

  protected <T> T lookup(final Class<T> role, final String hint) {
    return BeanFactoryAnnotationUtils.qualifiedBeanOfType(applicationContext.getAutowireCapableBeanFactory(), role,
        hint);
  }

  protected SecuritySystem getSecuritySystem() {
    return lookup(SecuritySystem.class);
  }

  protected UserManager getUserManager() {
    return lookup(UserManager.class);
  }

  protected final SecurityConfiguration getSecurityConfiguration() {
    return lookup(SecurityConfigurationSource.class, "default").getConfiguration();
  }

  public static class BaseSecurityConfiguration
  {
    @Qualifier("default")
    @Primary
    @Bean
    public SecurityConfigurationSource securityConfigurationSource() {
      return new PreconfiguredSecurityConfigurationSource(BaseSecurityConfig.get());
    }
  }

  @Configuration
  public static class TestSecurityConfigurationMocks
  {
    @Primary
    @Bean
    public EventManager eventManager() {
      return spy(new SimpleEventManager());
    }

    @Primary
    @Bean
    public AnonymousManager anonymousManager() {
      return mock(AnonymousManager.class);
    }

    @Qualifier("initial")
    @Primary
    @Bean
    public RealmConfiguration realmConfiguration() {
      RealmConfiguration realmConfiguration = new TestRealmConfiguration();
      realmConfiguration.setRealmNames(
          new ArrayList<>(Arrays.asList("MockRealmA", "MockRealmB", "MockRealmC", AuthorizingRealmImpl.NAME)));
      return realmConfiguration;
    }

    @Primary
    @Bean
    public NodeAccess nodeAccess() {
      return mock(NodeAccess.class);
    }

    @Primary
    @Bean
    public ApplicationDirectories applicationDirectories() {
      return mock(ApplicationDirectories.class);
    }

    /**
     * {@link org.globalart.astrum.swagger.internal.SwaggerModel} is component-scanned; its constructor needs this bean.
     */
    @Primary
    @Bean
    public ApplicationVersion applicationVersion() {
      ApplicationVersion applicationVersion = mock(ApplicationVersion.class);
      when(applicationVersion.getVersion()).thenReturn("0.0.0-test");
      return applicationVersion;
    }

    @Bean
    public DatabaseCheck databaseCheck() {
      return mock(DatabaseCheck.class);
    }

    @Bean
    public SecretsStore secretsStore() {
      return mock(SecretsStore.class);
    }

    @Bean
    public AuditRecorder auditRecorder() {
      return mock(AuditRecorder.class);
    }

    @Bean
    RealmConfigurationStore memoryRealmConfigurationStore() {
      return new MemoryRealmConfigurationStore();
    }
  }
}
