
package org.globalart.astrum.siesta;

import java.util.Map;

import jakarta.ws.rs.client.Client;
import jakarta.ws.rs.client.ClientBuilder;

import org.globalart.astrum.bootstrap.siesta.SiestaConfiguration;
import org.globalart.astrum.siesta.SiestaTestSupport.SiestaTestSupportConfiguration;

import org.eclipse.jetty.ee10.servlet.ServletHolder;
import org.eclipse.jetty.ee10.servlet.ServletTester;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.context.ApplicationContext;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.test.context.ContextConfiguration;

/**
 * Support for Siesta tests.
 */
@SpringBootTest
@ContextConfiguration(classes = {SiestaTestSupportConfiguration.class, SiestaConfiguration.class})
public abstract class SiestaTestSupport
{
  public static final String MOUNT_POINT = "/siesta";

  private ServletTester servletTester;

  private String url;

  private Client client;

  @Autowired
  protected ApplicationContext context;

  @ComponentScan({"org.globalart.astrum.siesta"})
  static class SiestaTestSupportConfiguration
  {
  }

  @BeforeEach
  void startJetty() throws Exception {
    servletTester = new ServletTester();
    servletTester.getContext();

    url = servletTester.createConnector(true) + MOUNT_POINT;

    ServletHolder holder = new ServletHolder(context.getBean(SiestaServlet.class));
    holder.setInitParameters(Map.of("resteasy.servlet.mapping.prefix", MOUNT_POINT));
    servletTester.addServlet(holder, MOUNT_POINT + "/*");

    servletTester.addServlet(DummyServlet.class, "/*");
    servletTester.start();

    client = ClientBuilder.newClient();
  }

  @AfterEach
  void stopJetty() throws Exception {
    if (servletTester != null) {
      servletTester.stop();
    }
  }

  protected Client client() {
    return client;
  }

  protected String url() {
    return url;
  }

  protected String url(final String path) {
    return url + "/" + path;
  }
}
