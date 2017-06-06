package test;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonDeserializer;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import java.io.IOException;
import java.util.Objects;
import java.util.Optional;

@JsonDeserialize(using = Type.Deserializer.class)
public class Type {
  private final String data;

  public Type(
    final String data
  ) {
    Objects.requireNonNull(data, "data");
    this.data = data;
  }

  public String getData() {
    return this.data;
  }

  @Override
  public int hashCode() {
    int result = 1;
    result = result * 31 + this.data.hashCode();
    return result;
  }

  @Override
  public boolean equals(final Object other) {
    if (other == null) {
      return false;
    }

    if (!(other instanceof Type)) {
      return false;
    }

    @SuppressWarnings("unchecked")
    final Type o = (Type) other;

    if (!this.data.equals(o.data)) {
      return false;
    }

    return true;
  }

  @Override
  public String toString() {
    final StringBuilder b = new StringBuilder();

    b.append("Type");
    b.append("(");
    b.append("data=");
    b.append(this.data.toString());
    b.append(")");

    return b.toString();
  }

  public static class Builder {
    private Optional<String> data = Optional.empty();

    public Builder data(final String data) {
      this.data = Optional.of(data);
      return this;
    }

    public Type build() {
      final String data = this.data.orElseThrow(() -> new RuntimeException("data: is required"));

      return new Type(data);
    }
  }

  public static class Model {
    private final String data;

    @JsonCreator
    public Model(
      @JsonProperty("data") final String data
    ) {
      this.data = data;
    }
  }

  public static class Deserializer extends JsonDeserializer<Type> {
    @Override
    public Type deserialize(final JsonParser parser, final DeserializationContext ctxt) throws IOException {
      final Model m = parser.readValueAs(Model.class);
      return new Type(m.data)
    }
  }
}
