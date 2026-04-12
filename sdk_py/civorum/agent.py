class Agent:
    """Minimal SDK surface for benchmark agents."""

    def act(self, observation, valid_actions):
        raise NotImplementedError("Agents must implement act().")

