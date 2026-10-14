"""Make a small chart from the preserved benchmark samples."""
import random
import statistics


def plot_results(datasets, root):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    from IPython.display import Image, display
    plt.rcParams.update({'font.size':11,'svg.fonttype':'none'})
    fig, axes = plt.subplots(1,2,figsize=(9,3.4),constrained_layout=True)
    variants = [('reference','Original','#65737e'),('cold_trie','Build trie each time','#b47736'),('warm_trie','Reuse trie','#267b9b')]
    for number,(axis,name,label) in enumerate(zip(axes,['json_record','three_digits'],['JSON record','Three digits'])):
        case = next(c for c in datasets['qwen']['cases'] if c['name']==name)
        original = case['reference']['samples_ms']
        for i,(key,text,color) in enumerate(variants):
            samples = case[key]['samples_ms']
            median = statistics.median(samples)
            assert median == case[key]['median_ms']
            rng = random.Random(20261014+number*10+i)
            boot = []
            for _ in range(2000):
                indexes = [rng.randrange(len(samples)) for _ in samples]
                boot.append(statistics.median(samples[j] for j in indexes))
            boot.sort()
            axis.barh(i,median,color=color,xerr=[[median-boot[49]],[boot[1949]-median]],capsize=3)
            axis.text(median+max(original)*.025,i,f'{median:.2f} ms',va='center',fontsize=10)
        axis.set_yticks(range(3),[v[1] for v in variants])
        axis.invert_yaxis()
        axis.set_title(label,loc='left')
        axis.set_xlim(0,max(case[k]['median_ms'] for k,_,_ in variants)*1.25)
        axis.set_xlabel('Index construction · milliseconds')
        axis.grid(axis='x',alpha=.15)
        axis.set_axisbelow(True)
        axis.spines[['top','right']].set_visible(False)
    fig.suptitle('Qwen vocabulary · preparation changes the answer',x=.03,ha='left')
    for extension in ['png']:
        fig.savefig(root/f'figures/preparation.{extension}',dpi=160)
    plt.close(fig)
    display(Image(filename=str(root/'figures/preparation.png')))
