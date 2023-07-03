class User:
    pass


class ProUser(User):
    pass


class TeamUser(User):
    pass


def make_new_user(user_class: type[User]) -> User:
    return user_class()
