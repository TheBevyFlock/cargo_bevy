/// A helper to make passing arguments to [`Command`](std::process::Command) more convenient.
#[derive(Debug, Clone)]
pub struct ArgBuilder(Vec<String>);

impl ArgBuilder {
    /// Create a new builder for command arguments.
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Add a singular argument.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_cli::external_cli::arg_builder::ArgBuilder;
    /// ArgBuilder::new().arg("--release");
    /// ```
    pub fn arg<A>(mut self, arg: A) -> Self
    where
        A: Into<String>,
    {
        self.0.push(arg.into());
        self
    }

    /// Add an argument with a value.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_cli::external_cli::arg_builder::ArgBuilder;
    /// ArgBuilder::new().add_with_value("--bin", "bevy");
    /// ```
    pub fn add_with_value<A, V>(self, arg: A, value: V) -> Self
    where
        A: Into<String>,
        V: Into<String>,
    {
        self.arg(arg).arg(value)
    }

    /// Add a boolean flag with the given name only if `value` is `true`.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_cli::external_cli::arg_builder::ArgBuilder;
    /// let is_release = true;
    /// ArgBuilder::new().add_flag_if("--release", is_release);
    /// ```
    pub fn add_flag_if<N>(self, name: N, value: bool) -> Self
    where
        N: Into<String>,
    {
        if value {
            self.arg(name)
        } else {
            self
        }
    }

    /// Add an argument with an optional value.
    ///
    /// If value is `&None`, no argument will be added.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_cli::external_cli::arg_builder::ArgBuilder;
    /// let maybe_name = Some("bevy");
    /// ArgBuilder::new().add_opt_value("--bin", &maybe_name);
    /// ```
    pub fn add_opt_value<N, V>(self, name: N, value: &Option<V>) -> Self
    where
        N: Into<String>,
        V: Into<String> + Clone,
    {
        if let Some(value) = value {
            self.add_with_value::<N, V>(name, value.clone())
        } else {
            self
        }
    }

    /// Add an argument with multiple values, separated by commas.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_cli::external_cli::arg_builder::ArgBuilder;
    /// let features = ["dev", "file_watcher"];
    /// ArgBuilder::new().add_value_list("--features", features);
    /// ```
    pub fn add_value_list<N, V>(self, name: N, value_list: impl IntoIterator<Item = V>) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        let values: Vec<String> = value_list.into_iter().map(|val| val.into()).collect();

        // If there are no values to add, omit the name of the argument as well
        if values.is_empty() {
            self
        } else {
            self.add_with_value(name, values.join(","))
        }
    }

    /// Add an argument with multiple values, reusing the same argument name.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_cli::external_cli::arg_builder::ArgBuilder;
    /// let features = ["dev", "file_watcher"];
    /// ArgBuilder::new().add_values_separately("--features", features);
    /// ```
    pub fn add_values_separately<N, V>(
        mut self,
        name: N,
        value_list: impl IntoIterator<Item = V>,
    ) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        let arg: String = name.into();

        for value in value_list.into_iter() {
            self = self.add_with_value(&arg, value);
        }

        self
    }

    /// Add all arguments from the other builder to this one.
    pub fn append(mut self, mut other: ArgBuilder) -> Self {
        self.0.append(&mut other.0);
        self
    }
}

impl Default for ArgBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for ArgBuilder {
    type Item = <Vec<String> as IntoIterator>::Item;
    type IntoIter = <Vec<String> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_args() {
        let args = ArgBuilder::new();
        assert_eq!(
            args.into_iter().collect::<Vec<String>>(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn arg_preserves_order() {
        let args = ArgBuilder::new().arg("one").arg("two").arg("three");
        assert_eq!(
            args.into_iter().collect::<Vec<String>>(),
            vec!["one", "two", "three"]
        );
    }

    #[test]
    fn add_with_value_adds_name_and_value() {
        let args = ArgBuilder::new().add_with_value("--bin", "bevy");
        assert_eq!(
            args.into_iter().collect::<Vec<String>>(),
            vec!["--bin", "bevy"]
        );
    }

    #[test]
    fn add_opt_value_adds_nothing_for_none() {
        let args = ArgBuilder::new().add_opt_value("--bin", &None::<String>);
        assert_eq!(
            args.into_iter().collect::<Vec<String>>(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn add_opt_value_adds_name_and_value_for_some() {
        let args = ArgBuilder::new().add_opt_value("--bin", &Some("bevy"));
        assert_eq!(
            args.into_iter().collect::<Vec<String>>(),
            vec!["--bin", "bevy"]
        );
    }

    #[test]
    fn add_flag_if_adds_flag_for_true() {
        let args = ArgBuilder::new().add_flag_if("--release", true);
        assert_eq!(args.into_iter().collect::<Vec<String>>(), vec!["--release"]);
    }

    #[test]
    fn add_flag_if_adds_flag_for_false() {
        let args = ArgBuilder::new().add_flag_if("--release", false);
        assert_eq!(
            args.into_iter().collect::<Vec<String>>(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn add_value_list_concatenates_values() {
        let args = ArgBuilder::new().add_value_list("--features", ["dev", "file_watcher"]);
        assert_eq!(
            args.into_iter().collect::<Vec<String>>(),
            vec!["--features", "dev,file_watcher"]
        );
    }

    #[test]
    fn add_value_list_empty_list_no_changes() {
        let args = ArgBuilder::new().add_value_list("--features", Vec::<String>::new());
        assert_eq!(
            args.into_iter().collect::<Vec<String>>(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn append_adds_args_after_self() {
        let args = ArgBuilder::new()
            .arg("one")
            .arg("two")
            .append(ArgBuilder::new().arg("three").arg("four"));
        assert_eq!(
            args.into_iter().collect::<Vec<String>>(),
            vec!["one", "two", "three", "four"]
        );
    }
}
