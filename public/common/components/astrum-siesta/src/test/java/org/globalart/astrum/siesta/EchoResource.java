
package org.globalart.astrum.siesta;

import java.util.List;

import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;

import org.globalart.astrum.rest.Resource;

import com.google.common.base.Function;
import com.google.common.collect.Lists;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.MediaType.APPLICATION_XML;
import org.springframework.stereotype.Component;

@Component
@Path("/echo")
public class EchoResource
    implements Resource
{
  @GET
  @Produces({APPLICATION_XML, APPLICATION_JSON})
  public List<String> get(
      @QueryParam("foo") String foo,
      @QueryParam("bar") Integer bar)
  {
    final List<String> result = Lists.newArrayList();
    if (foo != null) {
      result.add("foo=" + foo);
    }
    if (bar != null) {
      result.add("bar=" + bar);
    }
    return result;
  }

  @GET
  @Path("/multiple")
  @Produces({APPLICATION_XML, APPLICATION_JSON})
  public List<String> get(@QueryParam("foo") List<String> foo) {
    return Lists.transform(foo, new Function<String, String>()
    {
      public String apply(final String value) {
        return "foo=" + value;
      }
    });
  }
}
