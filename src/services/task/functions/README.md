# Functions of Task service

- **Retain**

Stores Point value on the disk if [input] is specified  
Returns stored value as a point, reads stored value only once

>**Example**
>
>```yaml
>fn Retain:
>    key: Ied13.Load
>    enable: const bool true
>    input: point real /App/Ied13/Load
>```
