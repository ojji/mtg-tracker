using UnityEngine;

namespace mtga_datacollector
{
  public class Loader
  {
    static GameObject _collector;
    static GameObject _dumper;

    public static void Load()
    {
      if (!GameObject.Find("mtga-datacollector"))
      {
        _collector = new GameObject("mtga-datacollector");
        _collector.AddComponent<MtgaDataCollector>();
        UnityEngine.Object.DontDestroyOnLoad(_collector);
      }
    }

    public static void LoadDumper(string directory)
    {
      if (!GameObject.Find("mtga-dumper"))
      {
        _dumper = new GameObject("mtga-dumper");
        _dumper.SetActive(false);
        var component = _dumper.AddComponent<MtgaDataDumper>();
        component.InitializeDirectory(directory);
        _dumper.SetActive(true);

        UnityEngine.Object.DontDestroyOnLoad(_dumper);
      }
    }

    public static void Unload()
    {
      if (_collector != null)
      {
        UnityEngine.Object.Destroy(_collector);
      }
    }
  }
}