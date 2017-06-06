import numbers

class Data:
  def __init__(self, name):
    self.name = name

  @staticmethod
  def decode(data):
    f_name = data["name"]

    return Data(f_name)

  def encode(self):
    data = dict()

    if self.name is None:
      raise Exception("name: is a required field")

    data["name"] = self.name

    return data

class Point:
  def __init__(self, timestamp, value):
    self.timestamp = timestamp
    self.value = value

  @staticmethod
  def decode(data):
    if data == 42:
      return Point(42, 41.2)

    if isinstance(data, numbers.Number):
      n = data
      return Point(n, 42)

    if isinstance(data, dict):
      p = Point.decode(data)
      return p

    f_timestamp = data[0]

    f_value = data[1]

    return Point(f_timestamp, f_value)

  def encode(self):
    if self.timestamp is None:
      raise Exception("timestamp: is a required field")

    if self.value is None:
      raise Exception("value: is a required field")

    return (self.timestamp, self.value)

class Interface:
  @staticmethod
  def decode(data):
    if isinstance(data, basestring):
      name = data
      return Interface_One(name, Data("data"))

    f_type = data["type"]

    if f_type == "one":
      return Interface_One.decode(data)

    if f_type == "two":
      return Interface_Two.decode(data)

    raise Exception("bad type" + f_type)

class Interface_One(Interface):
  TYPE = "one"

  def __init__(self, name, data):
    self.name = name
    self.data = data

  @staticmethod
  def decode(data):
    f_name = data["name"]

    f_data = Data.decode(data["data"])

    return Interface(f_name, f_data)

  def encode(self):
    data = dict()

    data["type"] = "one"

    if self.name is None:
      raise Exception("name: is a required field")

    data["name"] = self.name

    if self.data is None:
      raise Exception("data: is a required field")

    data["data"] = self.data.encode()

    return data

class Interface_Two(Interface):
  TYPE = "two"

  def __init__(self, name, data):
    self.name = name
    self.data = data

  @staticmethod
  def decode(data):
    f_name = data["name"]

    f_data = Data.decode(data["data"])

    return Interface(f_name, f_data)

  def encode(self):
    data = dict()

    data["type"] = "two"

    if self.name is None:
      raise Exception("name: is a required field")

    data["name"] = self.name

    if self.data is None:
      raise Exception("data: is a required field")

    data["data"] = self.data.encode()

    return data

class Type:
  def __init__(self, data):
    self.data = data

  @staticmethod
  def decode(data):
    if isinstance(data, basestring):
      data = data
      return Type(data)

    f_data = data["data"]

    return Type(f_data)

  def encode(self):
    data = dict()

    if self.data is None:
      raise Exception("data: is a required field")

    data["data"] = self.data

    return data
