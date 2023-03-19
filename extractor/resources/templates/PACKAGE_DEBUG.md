# `{{package.name.path_name}}`
## Version: `{{package.lock.version}}`

### Licensed Scripts

{% for path_name in licensed_scripts -%}
- `{{path_name}}`
{% endfor %}
### Unlicensed Scripts

{% for path_name in unlicensed_scripts -%}
- `{{path_name}}`
{% endfor %}

{% if is_blocked -%}
### Blocking Packages

{{blocking_tree}}
{% endif %}