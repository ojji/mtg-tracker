using UnityEngine;

namespace mtga_datacollector
{
  public class Loader
  {
    static GameObject gameObject;

    public static void Load()
    {
      if (!GameObject.Find("mtga-datacollector"))
      {
        gameObject = new GameObject("mtga-datacollector");
        gameObject.AddComponent<MtgaDataCollector>();
        Object.DontDestroyOnLoad(gameObject);
      }
    }

    public static void Unload()
    {
      Object.Destroy(gameObject);
    }
  }
}